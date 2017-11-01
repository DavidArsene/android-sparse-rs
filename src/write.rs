use std::cell::RefCell;
use std::cmp;
use std::fs::File as StdFile;
use std::io::{ErrorKind, Read, Seek, SeekFrom, Write};

use byteorder::{LittleEndian, WriteBytesExt};

use file::{Chunk, File};
use headers::{ChunkHeader, FileHeader};
use headers::BLOCK_SIZE;
use result::Result;

const COPY_BUF_SIZE: usize = 4096;

pub struct Writer<'a, W> {
    sparse_file: &'a mut File,
    // Use RefCell here, so we can iterate over the sparse_file chunks
    // while writing to dst.
    dst: RefCell<W>,
}

impl<'a, W: Write> Writer<'a, W> {
    pub fn new(sparse_file: &'a mut File, dst: W) -> Self {
        Self {
            sparse_file: sparse_file,
            dst: RefCell::new(dst),
        }
    }

    pub fn write(self) -> Result<()> {
        self.write_file_header()?;

        let chunks = { self.sparse_file.chunk_iter().collect::<Vec<_>>() };
        for chunk in chunks {
            self.write_chunk(chunk)?;
        }

        Ok(())
    }

    fn write_file_header(&self) -> Result<()> {
        let header = FileHeader {
            total_blocks: self.sparse_file.num_blocks(),
            total_chunks: self.sparse_file.num_chunks(),
            image_checksum: self.sparse_file.checksum(),
        };
        header.serialize(&mut *self.dst.borrow_mut())
    }

    fn write_chunk(&self, chunk: &Chunk) -> Result<()> {
        self.write_chunk_header(chunk)?;

        let dst = &mut *self.dst.borrow_mut();
        match *chunk {
            Chunk::Raw {
                ref file, offset, ..
            } => copy_from_file(file, dst, offset, chunk.raw_size() as usize)?,

            Chunk::Fill { ref fill, .. } => dst.write_all(fill)?,
            Chunk::DontCare { .. } => {}
            Chunk::Crc32 { crc } => dst.write_u32::<LittleEndian>(crc)?,
        }

        Ok(())
    }

    fn write_chunk_header(&self, chunk: &Chunk) -> Result<()> {
        let header = ChunkHeader {
            chunk_type: chunk.chunk_type(),
            chunk_size: chunk.raw_size() / BLOCK_SIZE,
            total_size: chunk.size(),
        };
        header.serialize(&mut *self.dst.borrow_mut())
    }
}

pub struct Decoder<'a, W> {
    sparse_file: &'a mut File,
    // Use RefCell here, so we can iterate over the sparse_file chunks
    // while writing to dst.
    dst: RefCell<W>,
}

impl<'a, W: Write> Decoder<'a, W> {
    pub fn new(sparse_file: &'a mut File, dst: W) -> Self {
        Self {
            sparse_file: sparse_file,
            dst: RefCell::new(dst),
        }
    }

    pub fn write(self) -> Result<()> {
        for chunk in self.sparse_file.chunk_iter() {
            self.write_chunk(chunk)?;
        }

        Ok(())
    }

    fn write_chunk(&self, chunk: &Chunk) -> Result<()> {
        let dst = &mut *self.dst.borrow_mut();
        match *chunk {
            Chunk::Raw {
                ref file, offset, ..
            } => copy_from_file(file, dst, offset, chunk.raw_size() as usize)?,

            Chunk::Fill { fill, num_blocks } => {
                let block = fill.iter()
                    .cycle()
                    .cloned()
                    .take(BLOCK_SIZE as usize)
                    .collect::<Vec<_>>();
                for _ in 0..num_blocks {
                    dst.write_all(&block)?;
                }
            }

            Chunk::DontCare { num_blocks } => {
                let block = [0; BLOCK_SIZE as usize];
                for _ in 0..num_blocks {
                    dst.write_all(&block)?;
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
