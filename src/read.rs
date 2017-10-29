use std::fs::File as StdFile;
use std::io::{ErrorKind, Read, Seek, SeekFrom};

use byteorder::{LittleEndian, ReadBytesExt};

use file::File;
use headers::{ChunkHeader, ChunkType, FileHeader};
use headers::BLOCK_SIZE;
use result::Result;

#[derive(Debug)]
pub struct Reader<'a> {
    sparse_file: &'a mut File,
}

impl<'a> Reader<'a> {
    pub fn new(sparse_file: &'a mut File) -> Self {
        Self { sparse_file }
    }

    pub fn read_from(&mut self, mut src: StdFile) -> Result<()> {
        self.sparse_file.set_backing_file(src.try_clone()?);

        let header = FileHeader::deserialize(&mut src)?;
        for _ in 0..header.total_chunks {
            self.read_chunk(&mut src)?;
        }

        Ok(())
    }

    fn read_chunk(&mut self, mut src: &mut StdFile) -> Result<()> {
        let header = ChunkHeader::deserialize(&mut src)?;
        let num_blocks = header.chunk_size;

        match header.chunk_type {
            ChunkType::Raw => {
                let off = src.seek(SeekFrom::Current(0))?;
                self.sparse_file.add_raw(off, num_blocks)?;
                let size = i64::from(num_blocks * BLOCK_SIZE);
                src.seek(SeekFrom::Current(size))?;
            }

            ChunkType::Fill => {
                let mut fill = [0; 4];
                src.read_exact(&mut fill)?;
                self.sparse_file.add_fill(fill, num_blocks)?;
            }

            ChunkType::DontCare => self.sparse_file.add_dont_care(num_blocks)?,

            ChunkType::Crc32 => {
                let crc = src.read_u32::<LittleEndian>()?;
                self.sparse_file.add_crc32(crc)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct Encoder<'a> {
    sparse_file: &'a mut File,
}

impl<'a> Encoder<'a> {
    pub fn new(sparse_file: &'a mut File) -> Self {
        Self { sparse_file }
    }

    pub fn read_from(&mut self, mut src: StdFile) -> Result<()> {
        self.sparse_file.set_backing_file(src.try_clone()?);

        let block_size = BLOCK_SIZE as usize;
        let mut block = vec![0; block_size];
        loop {
            let bytes_read = read_all(&mut src, &mut block)?;
            self.read_block(&block[..bytes_read], &mut src)?;
            if bytes_read != block_size {
                break;
            }
        }

        Ok(())
    }

    fn read_block(&mut self, block: &[u8], src: &mut StdFile) -> Result<()> {
        if block.is_empty() {
            return Ok(());
        }

        let num_blocks = 1;

        if is_sparse_block(block) {
            let mut fill = [0; 4];
            fill.copy_from_slice(&block[..4]);
            if fill == [0; 4] {
                self.sparse_file.add_dont_care(num_blocks)?;
            } else {
                self.sparse_file.add_fill(fill, num_blocks)?;
            }
            return Ok(());
        }

        let curr_off = src.seek(SeekFrom::Current(0))?;
        let off = curr_off - u64::from(BLOCK_SIZE);
        self.sparse_file.add_raw(off, num_blocks)?;

        Ok(())
    }
}

fn is_sparse_block(block: &[u8]) -> bool {
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
