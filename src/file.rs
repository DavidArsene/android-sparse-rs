use std::convert::TryInto;
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
        0
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
            Chunk::Raw { ref buf } => buf.len()
                .try_into()
                .expect("chunk size doesn't fit into u32"),
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
            Chunk::Fill { size, .. } | Chunk::DontCare { size } => size,
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
