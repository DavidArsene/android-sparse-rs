use std::fs::File as StdFile;
use std::slice::Iter;

use convert::TryInto;
use headers::ChunkType;
use headers::{BLOCK_SIZE, CHUNK_HEADER_SIZE};

pub type ChunkIter<'a> = Iter<'a, Chunk>;

#[derive(Debug, Default)]
pub struct File {
    chunks: Vec<Chunk>,
}

impl File {
    pub fn new() -> Self {
        Self { chunks: Vec::new() }
    }

    pub fn checksum(&self) -> u32 {
        // TODO
        0
    }

    pub fn num_blocks(&self) -> u32 {
        self.chunks
            .iter()
            .fold(0, |sum, chunk| sum + chunk.num_blocks())
    }

    pub fn num_chunks(&self) -> u32 {
        self.chunks
            .len()
            .try_into()
            .expect("number of chunks doesn't fit into u32")
    }

    pub fn add_chunk(&mut self, chunk: Chunk) {
        self.chunks.push(chunk);
    }

    pub fn chunk_iter(&self) -> ChunkIter {
        self.chunks.iter()
    }
}

#[derive(Debug)]
pub enum Chunk {
    Raw {
        file: StdFile,
        offset: u64,
        num_blocks: u32,
    },
    Fill { fill: [u8; 4], num_blocks: u32 },
    DontCare { num_blocks: u32 },
    Crc32 { crc: u32 },
}

impl Chunk {
    pub fn sparse_size(&self) -> u32 {
        let body_size = match *self {
            Chunk::Raw { num_blocks, .. } => num_blocks * BLOCK_SIZE,
            Chunk::Fill { .. } | Chunk::Crc32 { .. } => 4,
            Chunk::DontCare { .. } => 0,
        };
        u32::from(CHUNK_HEADER_SIZE) + body_size
    }

    pub fn num_blocks(&self) -> u32 {
        match *self {
            Chunk::Raw { num_blocks, .. } |
            Chunk::Fill { num_blocks, .. } |
            Chunk::DontCare { num_blocks } => num_blocks,
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
