//! Extensions for foreign types.

use crate::block::Block;
use crc32fast::Hasher;

/// Enables writing sparse blocks to `crc32fast::Hasher`s.
pub(crate) trait WriteBlock {
    fn write_block(&mut self, block: &Block);
}

impl WriteBlock for Hasher {
    fn write_block(&mut self, block: &Block) {
        match block {
            Block::Raw(buf) => self.update(&**buf),
            Block::Fill(value) => {
                for _ in 0..(Block::SIZE / 4) {
                    self.update(value);
                }
            }
            Block::Skip => self.update(&[0; Block::SIZE as usize]),
            Block::Crc32(_) => (),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tempfile::tempfile;
    use std::io::prelude::*;

    fn block_crc(block: &Block) -> u32 {
        let mut hasher = Hasher::new();
        hasher.write_block(block);
        hasher.finalize()
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
        assert_eq!(tmp.stream_position().unwrap(), 12);
    }
}
