//! Extensions for foreign types.

use std::io;
use std::io::prelude::*;
use std::io::SeekFrom;

use crc::crc32;
use crc::crc32::Hasher32;

use block::Block;

/// Enables writing sparse blocks to `crc::crc32::Digest`s.
pub trait WriteBlock {
    fn write_block(&mut self, block: &Block);
}

impl WriteBlock for crc32::Digest {
    fn write_block(&mut self, block: &Block) {
        match block {
            Block::Raw(buf) => self.write(&**buf),
            Block::Fill(value) => for _ in 0..(Block::SIZE / 4) {
                self.write(value);
            },
            Block::Skip => self.write(&[0; Block::SIZE as usize]),
            Block::Crc32(_) => (),
        }
    }
}

/// Enables conveniently getting the current offset of anything that
/// implements `Seek`.
pub trait Tell {
    fn tell(&mut self) -> io::Result<u64>;
}

impl<T: Seek> Tell for T {
    fn tell(&mut self) -> io::Result<u64> {
        self.seek(SeekFrom::Current(0))
    }
}
