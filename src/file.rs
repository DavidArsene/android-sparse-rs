use std::slice::Iter;

use headers::{ChunkHeader, ChunkType, FileHeader};
use headers::CHUNK_HEADER_SIZE;
use result::Result;

pub type ChunkIter<'a> = Iter<'a, Chunk>;

#[derive(Clone, Debug)]
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
        FileHeader {
            block_size: self.block_size,
            total_blocks: self.total_blocks(),
            total_chunks: self.chunks.len() as u32,
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

    pub fn add_chunk(&mut self, chunk: Chunk) -> Result<()> {
        if chunk.raw_size() % self.block_size != 0 {
            return Err("Chunk size must be multiple of block_size".into());
        }

        self.chunks.push(chunk);
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
        return 0;
    }
}

#[derive(Clone, Debug)]
pub enum Chunk {
    Raw { buf: Vec<u8> },
    Fill { fill: [u8; 4], size: u32 },
    DontCare { size: u32 },
    Crc32 { crc: [u8; 4] },
}

impl Chunk {
    pub fn size(&self) -> u32 {
        let body_size = match *self {
            Chunk::Raw { ref buf } => buf.len() as u32,
            Chunk::Fill { .. } => 4,
            Chunk::DontCare { .. } => 0,
            Chunk::Crc32 { .. } => 4,
        };
        CHUNK_HEADER_SIZE as u32 + body_size
    }

    pub fn raw_size(&self) -> u32 {
        match *self {
            Chunk::Raw { ref buf } => buf.len() as u32,
            Chunk::Fill { size, .. } => size,
            Chunk::DontCare { size } => size,
            Chunk::Crc32 { .. } => 0,
        }
    }

    pub fn chunk_type(&self) -> ChunkType {
        match *self {
            Chunk::Raw { .. } => ChunkType::Raw,
            Chunk::Fill { .. } => ChunkType::Fill,
            Chunk::DontCare { .. } => ChunkType::DontCare,
            Chunk::Crc32 { .. } => ChunkType::Crc32,
        }
    }
}
