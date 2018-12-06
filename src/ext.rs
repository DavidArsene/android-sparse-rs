//! Extensions for foreign types.

use crate::block::Block;
use crc::crc32::{self, Hasher32};
use std::io::{self, prelude::*, SeekFrom};

/// Enables writing sparse blocks to `crc::crc32::Digest`s.
pub(crate) trait WriteBlock {
    fn write_block(&mut self, block: &Block);
}

impl WriteBlock for crc32::Digest {
    fn write_block(&mut self, block: &Block) {
        match block {
            Block::Raw(buf) => self.write(&**buf),
            Block::Fill(value) => {
                for _ in 0..(Block::SIZE / 4) {
                    self.write(value);
                }
            }
            Block::Skip => self.write(&[0; Block::SIZE as usize]),
            Block::Crc32(_) => (),
        }
    }
}

/// Enables conveniently getting the current offset of anything that
/// implements `Seek`.
pub(crate) trait Tell {
    fn tell(&mut self) -> io::Result<u64>;
}

impl<T: Seek> Tell for T {
    fn tell(&mut self) -> io::Result<u64> {
        self.seek(SeekFrom::Current(0))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tempfile::tempfile;

    fn block_crc(block: &Block) -> u32 {
        let mut digest = crc32::Digest::new(crc32::IEEE);
        digest.write_block(block);
        digest.sum32()
    }

    #[test]
    fn crc_write_raw_block() {
        let block = Block::Raw(Box::new([b'A'; Block::SIZE as usize]));
        assert_eq!(block_crc(&block), 0xfea63440);
    }

    #[test]
    fn crc_write_fill_block() {
        let block = Block::Fill([b'A'; 4]);
        assert_eq!(block_crc(&block), 0xfea63440);
    }

    #[test]
    fn crc_write_skip_block() {
        let block = Block::Skip;
        assert_eq!(block_crc(&block), 0xc71c0011);
    }

    #[test]
    fn crc_write_crc32_block() {
        let block = Block::Crc32(0x12345678);
        assert_eq!(block_crc(&block), 0);
    }

    #[test]
    fn tell() {
        let mut tmp = tempfile().unwrap();
        writeln!(tmp, "hello world").unwrap();
        assert_eq!(tmp.tell().unwrap(), 12);
    }
}
