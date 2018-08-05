//! Sparse file reading and encoding.

use std::fs::File as StdFile;
use std::io::{ErrorKind, Read, Seek, SeekFrom};

use byteorder::{LittleEndian, ReadBytesExt};
use crc::crc32;
use crc::crc32::Hasher32;

use constants::BLOCK_SIZE;
use file::{Chunk, File};
use headers::{ChunkHeader, ChunkType, FileHeader};
use result::Result;

/// Reads sparse files from something that implements `Read`.
pub struct Reader {
    src: StdFile,
    crc: Option<crc32::Digest>,
}

impl Reader {
    /// Creates a new sparse file reader that reads from `src`.
    pub fn new(src: StdFile) -> Self {
        Self { src, crc: None }
    }

    /// Enables CRC32 checksum validation.
    pub fn with_crc(mut self) -> Self {
        self.crc = Some(crc32::Digest::new(crc32::IEEE));
        self
    }

    /// Reads `sparse_file` from this reader's source.
    pub fn read(mut self, mut sparse_file: &mut File) -> Result<()> {
        let header = FileHeader::deserialize(&mut self.src)?;
        for _ in 0..header.total_chunks {
            self.read_chunk(&mut sparse_file)?;
        }

        Ok(())
    }

    fn read_chunk(&mut self, mut spf: &mut File) -> Result<()> {
        let header = ChunkHeader::deserialize(&mut self.src)?;
        let num_blocks = header.chunk_size;

        match header.chunk_type {
            ChunkType::Raw => self.read_raw_chunk(&mut spf, num_blocks),
            ChunkType::Fill => self.read_fill_chunk(&mut spf, num_blocks),
            ChunkType::DontCare => self.read_dont_care_chunk(&mut spf, num_blocks),
            ChunkType::Crc32 => self.read_crc32_chunk(&mut spf),
        }
    }

    fn read_raw_chunk(&mut self, spf: &mut File, num_blocks: u32) -> Result<()> {
        let offset = self.src.seek(SeekFrom::Current(0))?;

        if let Some(ref mut digest) = self.crc {
            let mut block = [0; BLOCK_SIZE as usize];
            for _ in 0..num_blocks {
                self.src.read_exact(&mut block)?;
                digest.write(&block);
            }
        } else {
            let size = i64::from(num_blocks * BLOCK_SIZE);
            self.src.seek(SeekFrom::Current(size))?;
        }

        let chunk = Chunk::Raw {
            file: self.src.try_clone()?,
            offset,
            num_blocks,
        };
        spf.add_chunk(chunk);

        Ok(())
    }

    fn read_fill_chunk(&mut self, spf: &mut File, num_blocks: u32) -> Result<()> {
        let fill = read4(&mut self.src)?;

        if let Some(ref mut digest) = self.crc {
            for _ in 0..(num_blocks * BLOCK_SIZE / 4) {
                digest.write(&fill);
            }
        }

        let chunk = Chunk::Fill { fill, num_blocks };
        spf.add_chunk(chunk);

        Ok(())
    }

    fn read_dont_care_chunk(&mut self, spf: &mut File, num_blocks: u32) -> Result<()> {
        if let Some(ref mut digest) = self.crc {
            let block = [0; BLOCK_SIZE as usize];
            for _ in 0..num_blocks {
                digest.write(&block);
            }
        }

        let chunk = Chunk::DontCare { num_blocks };
        spf.add_chunk(chunk);

        Ok(())
    }

    fn read_crc32_chunk(&mut self, spf: &mut File) -> Result<()> {
        let crc = self.src.read_u32::<LittleEndian>()?;
        self.check_crc(crc)?;

        let chunk = Chunk::Crc32 { crc };
        spf.add_chunk(chunk);

        Ok(())
    }

    fn check_crc(&self, crc: u32) -> Result<()> {
        if let Some(ref digest) = self.crc {
            if digest.sum32() != crc {
                return Err("Checksum does not match".into());
            }
        }
        Ok(())
    }
}

/// Reads raw images from something that implements `Read` and encodes them
/// as sparse files.
pub struct Encoder {
    src: StdFile,
    chunk: Option<Chunk>,
}

impl Encoder {
    /// Creates a new sparse file encoder that reads from `src`.
    pub fn new(src: StdFile) -> Self {
        Self { src, chunk: None }
    }

    /// Reads a raw image from this encoder's source and encodes it into
    /// `sparse_file`.
    pub fn read(mut self, mut sparse_file: &mut File) -> Result<()> {
        let block_size = BLOCK_SIZE as usize;
        let mut block = vec![0; block_size];
        loop {
            let bytes_read = read_all(&mut self.src, &mut block)?;
            self.read_block(&mut sparse_file, &block[..bytes_read])?;
            if bytes_read != block_size {
                break;
            }
        }

        if let Some(last_chunk) = self.chunk.take() {
            sparse_file.add_chunk(last_chunk);
        }

        Ok(())
    }

    fn read_block(&mut self, spf: &mut File, block: &[u8]) -> Result<()> {
        if block.is_empty() {
            return Ok(());
        }

        if let Some(chunk) = self.merge_block(block)? {
            spf.add_chunk(chunk);
        }

        Ok(())
    }

    /// Tries to merge `block` into the current chunk. If this is not
    /// possible, returns the current chunk and makes `block` the new current
    /// chunk.
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

    /// Tries to merge the current chunk with the following raw block.
    fn merge_raw_block(&mut self) -> Result<Option<Chunk>> {
        let (old, new) = match self.chunk.take() {
            Some(Chunk::Raw {
                file,
                offset,
                num_blocks,
            }) => {
                let merged = Chunk::Raw {
                    file,
                    offset,
                    num_blocks: num_blocks + 1,
                };
                (None, merged)
            }
            old_chunk => {
                let mut file = self.src.try_clone()?;
                let curr_off = file.seek(SeekFrom::Current(0))?;
                let new_chunk = Chunk::Raw {
                    file,
                    offset: curr_off - u64::from(BLOCK_SIZE),
                    num_blocks: 1,
                };
                (old_chunk, new_chunk)
            }
        };
        self.chunk = Some(new);
        Ok(old)
    }

    /// Tries to merge the current chunk with the following block, filled with
    /// `fill`.
    fn merge_fill_block(&mut self, fill: [u8; 4]) -> Option<Chunk> {
        let new_fill = fill;
        let (old, new) = match self.chunk.take() {
            Some(Chunk::Fill { fill, num_blocks }) if fill == new_fill => (
                None,
                Chunk::Fill {
                    fill,
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

    // Tries to merge the current chunk with the following don't-care block.
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

/// Is `block` filled with the same 4-byte value?
fn is_sparse_block(block: &[u8]) -> bool {
    if block.len() != BLOCK_SIZE as usize {
        return false;
    }

    let mut words = block.chunks(4);
    let first = words.next().unwrap();
    for word in words {
        if word != first {
            return false;
        }
    }

    true
}

/// Reads 4 bytes from `r`.
fn read4<R: Read>(mut r: R) -> Result<[u8; 4]> {
    let mut buf = [0; 4];
    r.read_exact(&mut buf)?;
    Ok(buf)
}

/// Fills `buf` from `r`, returns an error if not possible.
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
