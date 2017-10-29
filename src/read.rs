use std::fs::File as StdFile;
use std::io::{ErrorKind, Read, Seek, SeekFrom};

use byteorder::{LittleEndian, ReadBytesExt};

use file::{Chunk, File};
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
        let header = FileHeader::deserialize(&mut src)?;
        for _ in 0..header.total_chunks {
            self.read_chunk(&mut src)?;
        }

        Ok(())
    }

    fn read_chunk(&mut self, mut src: &mut StdFile) -> Result<()> {
        let header = ChunkHeader::deserialize(&mut src)?;
        let num_blocks = header.chunk_size;

        let chunk = match header.chunk_type {
            ChunkType::Raw => {
                let off = src.seek(SeekFrom::Current(0))?;
                let size = i64::from(num_blocks * BLOCK_SIZE);
                src.seek(SeekFrom::Current(size))?;
                Chunk::Raw {
                    file: src.try_clone()?,
                    offset: off,
                    num_blocks: num_blocks,
                }
            }

            ChunkType::Fill => {
                let fill = read4(&mut src)?;
                Chunk::Fill { fill, num_blocks }
            }

            ChunkType::DontCare => Chunk::DontCare { num_blocks },

            ChunkType::Crc32 => Chunk::Crc32 {
                crc: src.read_u32::<LittleEndian>()?,
            },
        };
        self.sparse_file.add_chunk(chunk);
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

        let chunk = Self::chunk_from_block(block, src)?;
        self.sparse_file.add_chunk(chunk);

        Ok(())
    }

    fn chunk_from_block(block: &[u8], src: &StdFile) -> Result<Chunk> {
        let num_blocks = 1;
        let chunk = if is_sparse_block(block) {
            let fill = read4(block)?;
            if fill == [0; 4] {
                Chunk::DontCare { num_blocks }
            } else {
                Chunk::Fill { fill, num_blocks }
            }
        } else {
            let mut file = src.try_clone()?;
            let curr_off = file.seek(SeekFrom::Current(0))?;
            Chunk::Raw {
                file: file,
                offset: curr_off - u64::from(BLOCK_SIZE),
                num_blocks: num_blocks,
            }
        };
        Ok(chunk)
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

fn read4<R: Read>(mut r: R) -> Result<[u8; 4]> {
    let mut buf = [0; 4];
    r.read_exact(&mut buf)?;
    Ok(buf)
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
