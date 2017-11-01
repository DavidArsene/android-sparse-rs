use std::cmp;
use std::fs::File as StdFile;
use std::io::{ErrorKind, Read, Seek, SeekFrom, Write};

use byteorder::{LittleEndian, WriteBytesExt};

use file::{Chunk, File};
use headers::{ChunkHeader, FileHeader};
use headers::BLOCK_SIZE;
use result::Result;

const COPY_BUF_SIZE: usize = 4096;

pub struct Writer<W> {
    dst: W,
}

impl<W: Write> Writer<W> {
    pub fn new(dst: W) -> Self {
        Self { dst }
    }

    pub fn write(mut self, sparse_file: &File) -> Result<()> {
        self.write_file_header(sparse_file)?;

        for chunk in sparse_file.chunk_iter() {
            self.write_chunk(chunk)?;
        }

        Ok(())
    }

    fn write_file_header(&mut self, spf: &File) -> Result<()> {
        let header = FileHeader {
            total_blocks: spf.num_blocks(),
            total_chunks: spf.num_chunks(),
            image_checksum: spf.checksum(),
        };
        header.serialize(&mut self.dst)
    }

    fn write_chunk(&mut self, chunk: &Chunk) -> Result<()> {
        self.write_chunk_header(chunk)?;

        match *chunk {
            Chunk::Raw {
                ref file, offset, ..
            } => copy_from_file(file, &mut self.dst, offset, chunk.raw_size() as usize)?,

            Chunk::Fill { ref fill, .. } => self.dst.write_all(fill)?,
            Chunk::DontCare { .. } => {}
            Chunk::Crc32 { crc } => self.dst.write_u32::<LittleEndian>(crc)?,
        }

        Ok(())
    }

    fn write_chunk_header(&mut self, chunk: &Chunk) -> Result<()> {
        let header = ChunkHeader {
            chunk_type: chunk.chunk_type(),
            chunk_size: chunk.raw_size() / BLOCK_SIZE,
            total_size: chunk.size(),
        };
        header.serialize(&mut self.dst)
    }
}

pub struct Decoder<W> {
    dst: W,
}

impl<W: Write> Decoder<W> {
    pub fn new(dst: W) -> Self {
        Self { dst }
    }

    pub fn write(mut self, sparse_file: &File) -> Result<()> {
        for chunk in sparse_file.chunk_iter() {
            self.write_chunk(chunk)?;
        }

        Ok(())
    }

    fn write_chunk(&mut self, chunk: &Chunk) -> Result<()> {
        match *chunk {
            Chunk::Raw {
                ref file, offset, ..
            } => copy_from_file(file, &mut self.dst, offset, chunk.raw_size() as usize)?,

            Chunk::Fill { fill, num_blocks } => {
                let block = fill.iter()
                    .cycle()
                    .cloned()
                    .take(BLOCK_SIZE as usize)
                    .collect::<Vec<_>>();
                for _ in 0..num_blocks {
                    self.dst.write_all(&block)?;
                }
            }

            Chunk::DontCare { num_blocks } => {
                let block = [0; BLOCK_SIZE as usize];
                for _ in 0..num_blocks {
                    self.dst.write_all(&block)?;
                }
            }

            Chunk::Crc32 { .. } => (),
        };

        Ok(())
    }
}

fn copy_from_file<W: Write>(file: &StdFile, writer: W, offset: u64, size: usize) -> Result<()> {
    let mut file = file.try_clone()?;
    file.seek(SeekFrom::Start(offset))?;
    copy_exact(&mut file, writer, size)
}

fn copy_exact<R: Read, W: Write>(mut reader: R, mut writer: W, size: usize) -> Result<()> {
    let mut buf = [0; COPY_BUF_SIZE];

    let mut written = 0;
    while written != size {
        let read_size = cmp::min(size - written, COPY_BUF_SIZE);
        if let Err(err) = reader.read_exact(&mut buf[..read_size]) {
            if err.kind() == ErrorKind::Interrupted {
                continue;
            }
            return Err(err.into());
        }
        writer.write_all(&buf[..read_size])?;
        written += read_size;
    }

    Ok(())
}
