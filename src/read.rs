use std::fs::File as StdFile;
use std::io::{ErrorKind, Read, Seek, SeekFrom};

use byteorder::{LittleEndian, ReadBytesExt};

use file::{Chunk, File};
use headers::{ChunkHeader, ChunkType, FileHeader};
use headers::BLOCK_SIZE;
use result::Result;

pub struct Reader<'a> {
    sparse_file: &'a mut File,
    src: StdFile,
}

impl<'a> Reader<'a> {
    pub fn new(sparse_file: &'a mut File, src: StdFile) -> Self {
        Self { sparse_file, src }
    }

    pub fn read(mut self) -> Result<()> {
        let header = FileHeader::deserialize(&mut self.src)?;
        for _ in 0..header.total_chunks {
            self.read_chunk()?;
        }

        Ok(())
    }

    fn read_chunk(&mut self) -> Result<()> {
        let header = ChunkHeader::deserialize(&mut self.src)?;
        let num_blocks = header.chunk_size;

        let chunk = match header.chunk_type {
            ChunkType::Raw => {
                let off = self.src.seek(SeekFrom::Current(0))?;
                let size = i64::from(num_blocks * BLOCK_SIZE);
                self.src.seek(SeekFrom::Current(size))?;
                Chunk::Raw {
                    file: self.src.try_clone()?,
                    offset: off,
                    num_blocks: num_blocks,
                }
            }

            ChunkType::Fill => {
                let fill = read4(&mut self.src)?;
                Chunk::Fill { fill, num_blocks }
            }

            ChunkType::DontCare => Chunk::DontCare { num_blocks },

            ChunkType::Crc32 => Chunk::Crc32 {
                crc: self.src.read_u32::<LittleEndian>()?,
            },
        };
        self.sparse_file.add_chunk(chunk);
        Ok(())
    }
}

pub struct Encoder<'a> {
    sparse_file: &'a mut File,
    src: StdFile,
    chunk: Option<Chunk>,
}

impl<'a> Encoder<'a> {
    pub fn new(sparse_file: &'a mut File, src: StdFile) -> Self {
        Self {
            sparse_file: sparse_file,
            src: src,
            chunk: None,
        }
    }

    pub fn read(mut self) -> Result<()> {
        let block_size = BLOCK_SIZE as usize;
        let mut block = vec![0; block_size];
        loop {
            let bytes_read = read_all(&mut self.src, &mut block)?;
            self.read_block(&block[..bytes_read])?;
            if bytes_read != block_size {
                break;
            }
        }

        if let Some(last_chunk) = self.chunk.take() {
            self.sparse_file.add_chunk(last_chunk);
        }

        Ok(())
    }

    fn read_block(&mut self, block: &[u8]) -> Result<()> {
        if block.is_empty() {
            return Ok(());
        }

        if let Some(chunk) = self.merge_block(block)? {
            self.sparse_file.add_chunk(chunk);
        }

        Ok(())
    }

    fn merge_block(&mut self, block: &[u8]) -> Result<Option<Chunk>> {
        if is_sparse_block(block) {
            let fill = read4(block)?;
            if fill == [0; 4] {
                Ok(self.merge_don_care_block())
            } else {
                Ok(self.merge_fill_block(fill))
            }
        } else {
            self.merge_raw_block()
        }
    }

    fn merge_raw_block(&mut self) -> Result<Option<Chunk>> {
        let (old, new) = match self.chunk.take() {
            Some(Chunk::Raw {
                file,
                offset,
                num_blocks,
            }) => (
                None,
                Chunk::Raw {
                    file: file,
                    offset: offset,
                    num_blocks: num_blocks + 1,
                },
            ),
            old_chunk => {
                let mut file = self.src.try_clone()?;
                let curr_off = file.seek(SeekFrom::Current(0))?;
                (
                    old_chunk,
                    Chunk::Raw {
                        file: file,
                        offset: curr_off - u64::from(BLOCK_SIZE),
                        num_blocks: 1,
                    },
                )
            }
        };
        self.chunk = Some(new);
        Ok(old)
    }

    fn merge_fill_block(&mut self, fill: [u8; 4]) -> Option<Chunk> {
        let new_fill = fill;
        let (old, new) = match self.chunk.take() {
            Some(Chunk::Fill { fill, num_blocks }) if fill == new_fill => (
                None,
                Chunk::Fill {
                    fill: fill,
                    num_blocks: num_blocks + 1,
                },
            ),
            old_chunk => (
                old_chunk,
                Chunk::Fill {
                    fill: new_fill,
                    num_blocks: 1,
                },
            ),
        };
        self.chunk = Some(new);
        old
    }

    fn merge_don_care_block(&mut self) -> Option<Chunk> {
        let (old, new) = match self.chunk.take() {
            Some(Chunk::DontCare { num_blocks }) => (
                None,
                Chunk::DontCare {
                    num_blocks: num_blocks + 1,
                },
            ),
            old_chunk => (old_chunk, Chunk::DontCare { num_blocks: 1 }),
        };
        self.chunk = Some(new);
        old
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
