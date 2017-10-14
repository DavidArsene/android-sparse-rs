use std::slice::Iter;

use headers::{ChunkHeader, ChunkType, FileHeader};
use headers::CHUNK_HEADER_SIZE;
use result::Result;

pub type ChunkIter<'a> = Iter<'a, Chunk>;

#[derive(Clone, Debug)]
pub struct File {
    block_size: usize,
    chunks: Vec<Chunk>,
}

impl File {
    pub fn new(block_size: usize) -> Self {
        Self {
            block_size: block_size,
            chunks: Vec::new(),
        }
    }

    pub fn header(&self) -> FileHeader {
        FileHeader {
            block_size: self.block_size as u32,
            total_blocks: self.total_blocks() as u32,
            total_chunks: self.chunks.len() as u32,
            image_checksum: self.image_checksum(),
        }
    }

    pub fn chunk_header(&self, chunk: &Chunk) -> ChunkHeader {
        ChunkHeader {
            chunk_type: chunk.chunk_type(),
            chunk_size: (chunk.raw_size() / self.block_size) as u32,
            total_size: chunk.size() as u32,
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

    fn total_blocks(&self) -> usize {
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
    Fill { fill: [u8; 4], size: usize },
    DontCare { size: usize },
    Crc32 { crc: [u8; 4] },
}

impl Chunk {
    pub fn size(&self) -> usize {
        let body_size = match *self {
            Chunk::Raw { ref buf } => buf.len(),
            Chunk::Fill { .. } => 4,
            Chunk::DontCare { .. } => 0,
            Chunk::Crc32 { .. } => 4,
        };
        CHUNK_HEADER_SIZE + body_size
    }

    pub fn raw_size(&self) -> usize {
        match *self {
            Chunk::Raw { ref buf } => buf.len(),
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
