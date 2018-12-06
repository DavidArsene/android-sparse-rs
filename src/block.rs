//! A data structure for representing sparse blocks.

use std::fmt;

/// A sparse block and its associated data.
#[derive(Clone)]
pub enum Block {
    /// A raw block holding a byte buffer of length `Block::SIZE`.
    Raw(Box<[u8; Block::SIZE as usize]>),
    /// A fill block holding a 4-byte fill value.
    Fill([u8; 4]),
    /// A block that signifies a part of the image that can be skipped.
    Skip,
    /// A CRC32 block holding a checksum value.
    Crc32(u32),
}

impl Block {
    /// The size of a sparse file block.
    pub const SIZE: u32 = 4096;
}

impl fmt::Debug for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Block::*;

        match self {
            Raw(r) => write!(f, "Raw({:?})", &r[..]),
            Fill(_) | Skip | Crc32(_) => self.fmt(f),
        }
    }
}

impl PartialEq for Block {
    fn eq(&self, other: &Self) -> bool {
        use self::Block::*;

        match (self, other) {
            (Raw(r1), Raw(r2)) => r1[..] == r2[..],
            (Fill(v1), Fill(v2)) => v1 == v2,
            (Skip, Skip) => true,
            (Crc32(c1), Crc32(c2)) => c1 == c2,
            _ => false,
        }
    }
}
