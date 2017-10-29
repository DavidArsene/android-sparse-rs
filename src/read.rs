use std::fs::File as StdFile;
use std::io::{ErrorKind, Read, Seek, SeekFrom};

use byteorder::{LittleEndian, ReadBytesExt};

use file::File;
use headers::{ChunkHeader, ChunkType, FileHeader};
use headers::BLOCK_SIZE;
use result::Result;

#[derive(Debug)]
pub struct Reader {
    src: StdFile,
}

impl Reader {
    pub fn new(src: StdFile) -> Self {
        Self { src }
    }

    pub fn read(mut self) -> Result<File> {
        let header = FileHeader::deserialize(&mut self.src)?;

        let mut sparse_file = File::new();
        sparse_file.set_backing_file(self.src.try_clone()?);

        for _ in 0..header.total_chunks {
            self.read_chunk(&mut sparse_file)?;
        }

        Ok(sparse_file)
    }

    fn read_chunk(&mut self, sparse_file: &mut File) -> Result<()> {
        let header = ChunkHeader::deserialize(&mut self.src)?;
        let size = header.chunk_size * BLOCK_SIZE;

        match header.chunk_type {
            ChunkType::Raw => {
                let off = self.src.seek(SeekFrom::Current(0))?;
                sparse_file.add_raw(off, size)?;
                self.src.seek(SeekFrom::Current(i64::from(size)))?;
            }
            ChunkType::Fill => {
                let mut fill = [0; 4];
                self.src.read_exact(&mut fill)?;
                sparse_file.add_fill(fill, size)?;
            }
            ChunkType::DontCare => sparse_file.add_dont_care(size)?,
            ChunkType::Crc32 => {
                let crc = self.src.read_u32::<LittleEndian>()?;
                sparse_file.add_crc32(crc)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct Encoder {
    src: StdFile,
}

impl Encoder {
    pub fn new(src: StdFile) -> Self {
        Self { src }
    }

    pub fn read(mut self) -> Result<File> {
        let mut sparse_file = File::new();
        sparse_file.set_backing_file(self.src.try_clone()?);

        let block_size = BLOCK_SIZE as usize;
        let mut block = vec![0; block_size];
        loop {
            let bytes_read = read_all(&mut self.src, &mut block)?;
            self.read_block(&block[..bytes_read], &mut sparse_file)?;
            if bytes_read != block_size {
                break;
            }
        }

        Ok(sparse_file)
    }

    fn read_block(&mut self, block: &[u8], sparse_file: &mut File) -> Result<()> {
        if block.is_empty() {
            return Ok(());
        }

        if self.is_sparse_block(block) {
            let mut fill = [0; 4];
            fill.copy_from_slice(&block[..4]);
            if fill == [0; 4] {
                sparse_file.add_dont_care(BLOCK_SIZE)?;
            } else {
                sparse_file.add_fill(fill, BLOCK_SIZE)?;
            }
            return Ok(());
        }

        let curr_off = self.src.seek(SeekFrom::Current(0))?;
        let off = curr_off - u64::from(BLOCK_SIZE);
        sparse_file.add_raw(off, BLOCK_SIZE)?;

        Ok(())
    }

    fn is_sparse_block(&self, block: &[u8]) -> bool {
        if block.len() != BLOCK_SIZE as usize {
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
