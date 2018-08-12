//! A data structure for representing sparse blocks.

/// A sparse block and its associated data.
pub enum Block {
    /// A raw block holding a byte buffer of length `Block::SIZE`.
    Raw(Box<[u8; Block::SIZE as usize]>),
    /// A fill block holding a 4-byte fill value.
    Fill([u8; 4]),
    /// A block filled with null bytes.
    DontCare,
    /// A CRC32 block holding a checksum value.
    Crc32(u32),
}

impl Block {
    /// The size of a sparse file block.
    pub const SIZE: u32 = 4096;
}
