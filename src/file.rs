use std::fs::File as StdFile;
use std::slice::Iter;

use convert::TryInto;
use headers::ChunkType;
use headers::{BLOCK_SIZE, CHUNK_HEADER_SIZE};
use result::Result;

pub type ChunkIter<'a> = Iter<'a, Chunk>;

#[derive(Debug)]
pub struct File {
    backing_file: Option<StdFile>,
    chunks: Vec<Chunk>,
}

impl File {
    pub fn new() -> Self {
        Self {
            backing_file: None,
            chunks: Vec::new(),
        }
    }

    pub fn set_backing_file(&mut self, file: StdFile) {
        self.backing_file = Some(file);
    }

    pub fn checksum(&self) -> u32 {
        // TODO
        0
    }

    pub fn num_blocks(&self) -> u32 {
        self.chunks
            .iter()
            .fold(0, |sum, chunk| sum + chunk.raw_size() / BLOCK_SIZE)
    }

    pub fn num_chunks(&self) -> u32 {
        self.chunks
            .len()
            .try_into()
            .expect("number of chunks doesn't fit into u32")
    }

    pub fn add_raw(&mut self, offset: u64, size: u32) -> Result<()> {
        let backing_file = match self.backing_file {
            Some(ref f) => f,
            None => return Err("Sparse File not created with backing file".into()),
        };
        if size % BLOCK_SIZE != 0 {
            return Err("size must be multiple of block size".into());
        }

        let chunk = Chunk::Raw {
            file: backing_file.try_clone()?,
            offset: offset,
            size: size,
        };
        self.chunks.push(chunk);
        Ok(())
    }

    pub fn add_fill(&mut self, fill: [u8; 4], size: u32) -> Result<()> {
        if size % BLOCK_SIZE != 0 {
            return Err("size must be multiple of block size".into());
        }

        let chunk = Chunk::Fill { fill, size };
        self.chunks.push(chunk);
        Ok(())
    }

    pub fn add_dont_care(&mut self, size: u32) -> Result<()> {
        if size % BLOCK_SIZE != 0 {
            return Err("size must be multiple of block size".into());
        }

        let chunk = Chunk::DontCare { size };
        self.chunks.push(chunk);
        Ok(())
    }

    pub fn add_crc32(&mut self, crc: u32) -> Result<()> {
        let chunk = Chunk::Crc32 { crc };
        self.chunks.push(chunk);
        Ok(())
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
        size: u32,
    },
    Fill { fill: [u8; 4], size: u32 },
    DontCare { size: u32 },
    Crc32 { crc: u32 },
}

impl Chunk {
    pub fn size(&self) -> u32 {
        let body_size = match *self {
            Chunk::Raw { size, .. } => size,
            Chunk::Fill { .. } | Chunk::Crc32 { .. } => 4,
            Chunk::DontCare { .. } => 0,
        };
        u32::from(CHUNK_HEADER_SIZE) + body_size
    }

    pub fn raw_size(&self) -> u32 {
        match *self {
            Chunk::Raw { size, .. } | Chunk::Fill { size, .. } | Chunk::DontCare { size } => size,
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
