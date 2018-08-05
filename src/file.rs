//! Sparse file data structures.

use std::fs::File as StdFile;
use std::slice::Iter;

use constants::{BLOCK_SIZE, CHUNK_HEADER_SIZE};
use convert::TryInto;

/// An iterator over sparse file chunks.
pub type ChunkIter<'a> = Iter<'a, Chunk>;

/// A sparse file.
///
/// Provides methods to add chunks and iterate over its chunks.
#[derive(Debug, Default)]
pub struct File {
    chunks: Vec<Chunk>,
}

impl File {
    /// Creates a new empty sparse file.
    pub fn new() -> Self {
        Self { chunks: Vec::new() }
    }

    /// Returns the number of blocks in the sparse file.
    pub fn num_blocks(&self) -> u32 {
        self.chunks
            .iter()
            .fold(0, |sum, chunk| sum + chunk.num_blocks())
    }

    /// Returns the number of chunks in the sparse file.
    pub fn num_chunks(&self) -> u32 {
        self.chunks
            .len()
            .try_into()
            .expect("number of chunks doesn't fit into u32")
    }

    /// Adds a chunk to the sparse file.
    pub fn add_chunk(&mut self, chunk: Chunk) {
        self.chunks.push(chunk);
    }

    /// Iterates over the chunks in the sparse file.
    pub fn chunk_iter(&self) -> ChunkIter {
        self.chunks.iter()
    }
}

/// A sparse file chunk, representing either:
/// - a part of the raw image of size `num_blocks * BLOCK_SIZE`, or
/// - the CRC32 checksum of the raw image at this point
#[derive(Debug)]
pub enum Chunk {
    /// Chunk representing blocks of raw bytes.
    ///
    /// To keep memory usage low when dealing with sparse files, a raw chunk
    /// holds a reference to its backing file instead of the actual data.
    Raw {
        /// Reference to the backing file.
        file: StdFile,
        /// Offset into the backing file to the start of the raw bytes.
        offset: u64,
        /// Number of blocks contained in this chunk.
        num_blocks: u32,
    },

    /// Chunk representing blocks filled with the same 4-byte value.
    Fill {
        /// The 4-byte fill value.
        fill: [u8; 4],
        /// Number of blocks contained in this chunk.
        num_blocks: u32,
    },

    /// Chunk representing blocks of ignored data.
    DontCare {
        /// Number of blocks contained in this chunk.
        num_blocks: u32,
    },

    /// Chunk holding the CRC32 checksum value of all previous data chunks.
    Crc32 {
        /// The CRC32 checksum value.
        crc: u32,
    },
}

impl Chunk {
    /// Returns the chunk's size in a sparse file.
    pub fn sparse_size(&self) -> u32 {
        let body_size = match *self {
            Chunk::Raw { num_blocks, .. } => num_blocks * BLOCK_SIZE,
            Chunk::Fill { .. } | Chunk::Crc32 { .. } => 4,
            Chunk::DontCare { .. } => 0,
        };
        u32::from(CHUNK_HEADER_SIZE) + body_size
    }

    /// Returns the size of the chunk's decoded raw data.
    pub fn raw_size(&self) -> u32 {
        self.num_blocks() * BLOCK_SIZE
    }

    /// Returns the number of blocks represented by the chunk.
    pub fn num_blocks(&self) -> u32 {
        match *self {
            Chunk::Raw { num_blocks, .. }
            | Chunk::Fill { num_blocks, .. }
            | Chunk::DontCare { num_blocks } => num_blocks,
            Chunk::Crc32 { .. } => 0,
        }
    }
}
