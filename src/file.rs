use std::convert::TryInto;
use std::fs::File as StdFile;
use std::slice::Iter;

use headers::{ChunkHeader, ChunkType, FileHeader};
use headers::CHUNK_HEADER_SIZE;
use result::Result;

pub type ChunkIter<'a> = Iter<'a, Chunk>;

#[derive(Debug)]
pub struct File {
    block_size: u32,
    chunks: Vec<Chunk>,
}

impl File {
    pub fn new(block_size: u32) -> Self {
        Self {
            block_size: block_size,
            chunks: Vec::new(),
        }
    }

    pub fn header(&self) -> FileHeader {
        let total_chunks = self.chunks
            .len()
            .try_into()
            .expect("number of chunks doesn't fit into u32");

        FileHeader {
            block_size: self.block_size,
            total_blocks: self.total_blocks(),
            total_chunks: total_chunks,
            image_checksum: self.image_checksum(),
        }
    }

    pub fn chunk_header(&self, chunk: &Chunk) -> ChunkHeader {
        ChunkHeader {
            chunk_type: chunk.chunk_type(),
            chunk_size: chunk.raw_size() / self.block_size,
            total_size: chunk.size(),
        }
    }

    pub fn add_raw(&mut self, buf: &[u8]) -> Result<()> {
        if buf.len() % self.block_size as usize != 0 {
            return Err("bytes size must be multiple of block_size".into());
        }

        let new_buf = buf;
        if let Some(&mut Chunk::Raw { ref mut buf }) = self.chunks.iter_mut().last() {
            buf.extend(new_buf.iter().cloned());
            return Ok(());
        }

        let buf = new_buf.to_vec();
        self.chunks.push(Chunk::Raw { buf });
        Ok(())
    }

    pub fn add_fill(&mut self, fill: [u8; 4], size: u32) -> Result<()> {
        if size % self.block_size != 0 {
            return Err("size must be multiple of block_size".into());
        }

        let (new_fill, new_size) = (fill, size);
        if let Some(&mut Chunk::Fill { fill, ref mut size }) = self.chunks.iter_mut().last() {
            if fill == new_fill {
                *size += new_size;
                return Ok(());
            }
        }

        self.chunks.push(Chunk::Fill { fill, size });
        Ok(())
    }

    pub fn add_dont_care(&mut self, size: u32) -> Result<()> {
        if size % self.block_size != 0 {
            return Err("size must be multiple of block_size".into());
        }

        let new_size = size;
        if let Some(&mut Chunk::DontCare { ref mut size }) = self.chunks.iter_mut().last() {
            *size += new_size;
            return Ok(());
        }

        self.chunks.push(Chunk::DontCare { size });
        Ok(())
    }

    pub fn add_crc32(&mut self, crc: u32) -> Result<()> {
        self.chunks.push(Chunk::Crc32 { crc });
        Ok(())
    }

    pub fn chunk_iter(&self) -> ChunkIter {
        self.chunks.iter()
    }

    fn total_blocks(&self) -> u32 {
        self.chunks
            .iter()
            .fold(0, |sum, chunk| sum + chunk.raw_size() / self.block_size)
    }

    fn image_checksum(&self) -> u32 {
        // TODO
        0
    }
}

#[derive(Debug)]
pub enum Chunk {
    Raw { buf: Vec<u8> },
    RawFileBacked {
        file: StdFile,
        offset: u64,
        size: u32,
    },
    Fill { fill: [u8; 4], size: u32 },
    DontCare { size: u32 },
    Crc32 { crc: u32 },
}

impl Chunk {
    pub fn size(&self) -> u32 {
        let body_size = match *self {
            Chunk::Raw { ref buf } => buf.len()
                .try_into()
                .expect("chunk size doesn't fit into u32"),
            Chunk::RawFileBacked { size, .. } => size,
            Chunk::Fill { .. } | Chunk::Crc32 { .. } => 4,
            Chunk::DontCare { .. } => 0,
        };
        u32::from(CHUNK_HEADER_SIZE) + body_size
    }

    pub fn raw_size(&self) -> u32 {
        match *self {
            Chunk::Raw { ref buf } => buf.len()
                .try_into()
                .expect("raw chunk size doesn't fit into u32"),
            Chunk::RawFileBacked { size, .. } |
            Chunk::Fill { size, .. } |
            Chunk::DontCare { size } => size,
            Chunk::Crc32 { .. } => 0,
        }
    }

    pub fn chunk_type(&self) -> ChunkType {
        match *self {
            Chunk::Raw { .. } | Chunk::RawFileBacked { .. } => ChunkType::Raw,
            Chunk::Fill { .. } => ChunkType::Fill,
            Chunk::DontCare { .. } => ChunkType::DontCare,
            Chunk::Crc32 { .. } => ChunkType::Crc32,
        }
    }
}
