use std::fs::File as StdFile;
use std::io::{Read, Seek, SeekFrom, Write};

use byteorder::{LittleEndian, WriteBytesExt};
use crc::crc32;
use crc::crc32::Hasher32;

use constants::BLOCK_SIZE;
use file::{Chunk, File};
use headers::{ChunkHeader, ChunkType, FileHeader};
use result::Result;

pub struct Writer<W> {
    dst: W,
    crc: Option<crc32::Digest>,
}

impl<W: Write> Writer<W> {
    pub fn new(dst: W) -> Self {
        Self { dst, crc: None }
    }

    pub fn with_crc(mut self) -> Self {
        self.crc = Some(crc32::Digest::new(crc32::IEEE));
        self
    }

    pub fn write(mut self, sparse_file: &File) -> Result<()> {
        self.write_file_header(sparse_file)?;

        for chunk in sparse_file.chunk_iter() {
            self.write_chunk(chunk)?;
        }

        self.write_end_chunk()
    }

    fn write_file_header(&mut self, spf: &File) -> Result<()> {
        let mut total_chunks = spf.num_chunks();
        if self.crc.is_some() {
            total_chunks += 1;
        }

        let header = FileHeader {
            total_blocks: spf.num_blocks(),
            total_chunks,
            image_checksum: 0,
        };
        header.serialize(&mut self.dst)
    }

    fn write_chunk(&mut self, chunk: &Chunk) -> Result<()> {
        self.write_chunk_header(chunk)?;

        match *chunk {
            Chunk::Raw {
                ref file,
                offset,
                num_blocks,
            } => self.write_raw_chunk(file, offset, num_blocks),
            Chunk::Fill { fill, num_blocks } => self.write_fill_chunk(fill, num_blocks),
            Chunk::DontCare { num_blocks } => self.write_dont_care_chunk(num_blocks),
            Chunk::Crc32 { crc } => self.write_crc32_chunk(crc),
        }
    }

    fn write_chunk_header(&mut self, chunk: &Chunk) -> Result<()> {
        let chunk_type = match *chunk {
            Chunk::Raw { .. } => ChunkType::Raw,
            Chunk::Fill { .. } => ChunkType::Fill,
            Chunk::DontCare { .. } => ChunkType::DontCare,
            Chunk::Crc32 { .. } => ChunkType::Crc32,
        };
        let header = ChunkHeader {
            chunk_type,
            chunk_size: chunk.num_blocks(),
            total_size: chunk.sparse_size(),
        };
        header.serialize(&mut self.dst)
    }

    fn write_raw_chunk(&mut self, file: &StdFile, offset: u64, num_blocks: u32) -> Result<()> {
        let mut file = file.try_clone()?;
        file.seek(SeekFrom::Start(offset))?;

        if let Some(ref mut digest) = self.crc {
            let mut block = [0; BLOCK_SIZE as usize];
            for _ in 0..num_blocks {
                file.read_exact(&mut block)?;
                digest.write(&block);
                self.dst.write_all(&block)?;
            }
        } else {
            copy_blocks(&mut file, &mut self.dst, num_blocks)?;
        }

        Ok(())
    }

    fn write_fill_chunk(&mut self, fill: [u8; 4], num_blocks: u32) -> Result<()> {
        if let Some(ref mut digest) = self.crc {
            for _ in 0..(num_blocks * BLOCK_SIZE / 4) {
                digest.write(&fill);
            }
        }

        self.dst.write_all(&fill).map_err(|e| e.into())
    }

    fn write_dont_care_chunk(&mut self, num_blocks: u32) -> Result<()> {
        if let Some(ref mut digest) = self.crc {
            let block = [0; BLOCK_SIZE as usize];
            for _ in 0..num_blocks {
                digest.write(&block);
            }
        }

        Ok(())
    }

    fn write_crc32_chunk(&mut self, crc: u32) -> Result<()> {
        self.dst
            .write_u32::<LittleEndian>(crc)
            .map_err(|e| e.into())
    }

    fn write_end_chunk(&mut self) -> Result<()> {
        let crc = self.crc.take();
        if let Some(digest) = crc {
            let chunk = Chunk::Crc32 {
                crc: digest.sum32(),
            };
            self.write_chunk(&chunk)
        } else {
            Ok(())
        }
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
                ref file,
                offset,
                num_blocks,
            } => copy_from_file(file, &mut self.dst, offset, num_blocks)?,

            Chunk::Fill { fill, num_blocks } => {
                let block = fill
                    .iter()
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

fn copy_from_file<W: Write>(file: &StdFile, writer: W, offset: u64, num_blocks: u32) -> Result<()> {
    let mut file = file.try_clone()?;
    file.seek(SeekFrom::Start(offset))?;
    copy_blocks(&mut file, writer, num_blocks)
}

fn copy_blocks<R: Read, W: Write>(mut r: R, mut w: W, num_blocks: u32) -> Result<()> {
    let mut block = [0; BLOCK_SIZE as usize];
    for _ in 0..num_blocks {
        r.read_exact(&mut block)?;
        w.write_all(&block)?;
    }

    Ok(())
}
