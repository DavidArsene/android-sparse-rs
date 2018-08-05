//! Public constants.

/// Size of the sparse file header.
pub const FILE_HEADER_SIZE: u16 = 28;

/// Size of the sparse file chunk header.
pub const CHUNK_HEADER_SIZE: u16 = 12;

/// Size of a sparse file block.
pub const BLOCK_SIZE: u32 = 4096;
