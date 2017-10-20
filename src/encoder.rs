use std::io::{ErrorKind, Read};

use file::File;
use result::Result;

#[derive(Clone, Debug)]
pub struct Encoder<R> {
    r: R,
    block_size: u32,
}

impl<R: Read> Encoder<R> {
    pub fn new(r: R, block_size: u32) -> Self {
        Self { r, block_size }
    }

    pub fn read(mut self) -> Result<File> {
        let mut sparse_file = File::new(self.block_size);
        let block_size = self.block_size as usize;

        let mut block = vec![0; block_size];
        loop {
            let bytes_read = read_all(&mut self.r, &mut block)?;
            self.read_block(&block[..bytes_read], &mut sparse_file)?;
            if bytes_read != block_size {
                break;
            }
        }

        Ok(sparse_file)
    }

    fn read_block(&self, block: &[u8], sparse_file: &mut File) -> Result<()> {
        if block.is_empty() {
            return Ok(());
        }

        if self.is_sparse_block(block) {
            let mut fill = [0; 4];
            fill.copy_from_slice(&block[..4]);
            if fill == [0; 4] {
                sparse_file.add_dont_care(self.block_size)?;
            } else {
                sparse_file.add_fill(fill, self.block_size)?;
            }
        } else {
            sparse_file.add_raw(block)?;
        }

        Ok(())
    }

    fn is_sparse_block(&self, block: &[u8]) -> bool {
        if block.len() != self.block_size as usize {
            return false;
        }

        let mut words = block.chunks(4);
        let first = words.next().expect("block is empty");
        for word in words {
            if word != first {
                return false;
            }
        }

        true
    }
}

fn read_all<R: Read>(mut r: R, mut buf: &mut [u8]) -> Result<usize> {
    let buf_size = buf.len();

    while !buf.is_empty() {
        match r.read(buf) {
            Ok(0) => break,
            Ok(n) => {
                let tmp = buf;
                buf = &mut tmp[n..]
            }
            Err(ref e) if e.kind() == ErrorKind::Interrupted => (),
            Err(e) => return Err(e.into()),
        };
    }

    Ok(buf_size - buf.len())
}
