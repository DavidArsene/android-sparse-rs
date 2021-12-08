//! Sparse image reading and encoding from raw images.

use crate::{
    block::Block,
    ext::WriteBlock,
    headers::{ChunkHeader, ChunkType, FileHeader},
    result::{Error, Result},
};
use byteorder::{LittleEndian, ReadBytesExt};
use crc::crc32::{self, Hasher32};
use std::{
    io::{prelude::*, BufReader, ErrorKind},
    mem, slice,
};

const BLOCK_SIZE: usize = Block::SIZE as usize;
const U32_BLOCK_SIZE: usize = BLOCK_SIZE / mem::size_of::<u32>();

/// Reads sparse blocks from a sparse image.
///
/// Implements the `Iterator` trait, so sparse blocks can be read from
/// a reader by iterating over it.
pub struct Reader<R: Read> {
    src: BufReader<R>,
    current_chunk: Option<ChunkHeader>,
    current_fill: Option<[u8; 4]>,
    remaining_chunks: u32,
    crc: Option<crc32::Digest>,
    finished: bool,
}

impl<R: Read> Reader<R> {
    /// Creates a new reader that reads from `r`.
    ///
    /// The created reader skips checksum verification in favor of
    /// speed. To get a reader that does checksum verification, use
    /// `Reader::with_crc` instead.
    pub fn new(r: R) -> Result<Self> {
        let mut src = BufReader::new(r);
        let header = FileHeader::read_from(&mut src)?;
        Ok(Self {
            src,
            current_chunk: None,
            current_fill: None,
            remaining_chunks: header.total_chunks,
            crc: None,
            finished: false,
        })
    }

    /// Creates a new reader that reads from `r` and verifies all
    /// included checksums.
    pub fn with_crc(r: R) -> Result<Self> {
        let mut reader = Self::new(r)?;
        reader.crc = Some(crc32::Digest::new(crc32::IEEE));
        Ok(reader)
    }

    fn next_block(&mut self) -> Result<Block> {
        let mut chunk = match self.current_chunk.take() {
            Some(c) => c,
            None => ChunkHeader::read_from(&mut self.src)?,
        };

        let block = self.read_block(&mut chunk)?;
        if let Some(digest) = self.crc.as_mut() {
            digest.write_block(&block);
        }

        if chunk.chunk_size <= 1 {
            self.remaining_chunks -= 1;
            self.current_chunk = None;
            self.current_fill = None;
        } else {
            chunk.chunk_size -= 1;
            self.current_chunk = Some(chunk);
        }

        Ok(block)
    }

    fn read_block(&mut self, chunk: &mut ChunkHeader) -> Result<Block> {
        match chunk.chunk_type {
            ChunkType::Raw => {
                let mut buf = [0; BLOCK_SIZE];
                self.src.read_exact(&mut buf)?;
                Ok(Block::Raw(Box::new(buf)))
            }
            ChunkType::Fill => {
                let value = match self.current_fill {
                    Some(v) => v,
                    None => {
                        self.current_fill = Some(read4(&mut self.src)?);
                        self.current_fill.unwrap()
                    }
                };
                Ok(Block::Fill(value))
            }
            ChunkType::DontCare => Ok(Block::Skip),
            ChunkType::Crc32 => {
                let checksum = self.src.read_u32::<LittleEndian>()?;
                self.verify_checksum(checksum)?;
                Ok(Block::Crc32(checksum))
            }
        }
    }

    fn verify_checksum(&self, checksum: u32) -> Result<()> {
        if let Some(digest) = self.crc.as_ref() {
            if digest.sum32() != checksum {
                return Err(Error::Parse("Checksum does not match".into()));
            }
        }

        Ok(())
    }
}

impl<R: Read> Iterator for Reader<R> {
    type Item = Result<Block>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let result = self.next_block();
        self.finished = result.is_err() || self.remaining_chunks == 0;
        Some(result)
    }
}

fn read4<R: Read>(mut r: R) -> Result<[u8; 4]> {
    let mut buf = [0; 4];
    r.read_exact(&mut buf)?;
    Ok(buf)
}

/// Wraps a block-sized buffer that is guaranteed to be 4-byte aligned.
///
/// This allows it to give out `&[u8]` and `&[u32]` views of the
/// buffer. We need both kinds of views, since efficient checking for
/// sparse blocks requires `&[u32]` while reading and writing only
/// works on `&[u8]`.
struct AlignedBuf([u32; U32_BLOCK_SIZE]);

impl AlignedBuf {
    fn new() -> Self {
        AlignedBuf([0; U32_BLOCK_SIZE])
    }

    fn as_ref(&self) -> &[u8] {
        let ptr = self.0.as_ptr().cast();
        let len = self.0.len() * mem::size_of::<u32>();
        unsafe { slice::from_raw_parts(ptr, len) }
    }

    fn as_mut(&mut self) -> &mut [u8] {
        let ptr = self.0.as_mut_ptr().cast();
        let len = self.0.len() * mem::size_of::<u32>();
        unsafe { slice::from_raw_parts_mut(ptr, len) }
    }

    fn as_u32(&self) -> &[u32] {
        &self.0
    }

    fn into_inner(self) -> [u8; BLOCK_SIZE] {
        unsafe { mem::transmute::<[u32; U32_BLOCK_SIZE], [u8; BLOCK_SIZE]>(self.0) }
    }
}

/// Reads blocks from a raw image and encodes them into sparse blocks.
///
/// Implements the `Iterator` trait, so sparse blocks can be read from
/// an encoder by iterating over it.
pub struct Encoder<R: Read> {
    src: R,
    finished: bool,
}

impl<R: Read> Encoder<R> {
    /// Creates a new encoder that reads from `r`.
    pub fn new(r: R) -> Result<Self> {
        Ok(Self {
            src: r,
            finished: false,
        })
    }

    fn read_block(&mut self) -> Result<Option<Block>> {
        let mut buf = AlignedBuf::new();
        let bytes_read = read_all(&mut self.src, buf.as_mut())?;

        let block = match bytes_read {
            0 => None,
            _ => Some(Self::encode_block(buf)),
        };
        Ok(block)
    }

    fn encode_block(buf: AlignedBuf) -> Block {
        if is_sparse(buf.as_u32()) {
            let value = read4(buf.as_ref()).unwrap();
            if value == [0; 4] {
                Block::Skip
            } else {
                Block::Fill(value)
            }
        } else {
            Block::Raw(Box::new(buf.into_inner()))
        }
    }
}

impl<R: Read> Iterator for Encoder<R> {
    type Item = Result<Block>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        match self.read_block() {
            Ok(Some(c)) => Some(Ok(c)),
            Ok(None) => {
                self.finished = true;
                None
            }
            Err(e) => {
                self.finished = true;
                Some(Err(e))
            }
        }
    }
}

fn read_all<R: Read>(mut r: R, mut buf: &mut [u8]) -> Result<usize> {
    let buf_size = buf.len();

    while !buf.is_empty() {
        match r.read(buf) {
            Ok(0) => break,
            Ok(n) => {
                let tmp = buf;
                buf = &mut tmp[n..];
            }
            Err(ref e) if e.kind() == ErrorKind::Interrupted => (),
            Err(e) => return Err(e.into()),
        };
    }

    Ok(buf_size - buf.len())
}

fn is_sparse(buf: &[u32]) -> bool {
    let mut parts = buf.iter();
    let first = parts.next().unwrap();
    parts.all(|p| p == first)
}

#[cfg(test)]
mod test {
    use super::*;

    const U8_BUF: &[u8] = &[0xaa; BLOCK_SIZE];
    const U32_BUF: &[u32] = &[0xaaaaaaaa; U32_BLOCK_SIZE];
    const HALF_BLOCK_SIZE: usize = BLOCK_SIZE / 2;

    #[test]
    fn aligned_buf() {
        let mut buf = AlignedBuf::new();
        buf.as_mut().write_all(U8_BUF).unwrap();

        assert_eq!(buf.as_ref(), U8_BUF);
        assert_eq!(buf.as_u32(), U32_BUF);

        let content = buf.into_inner();
        assert_eq!(&content[..], U8_BUF);
    }

    #[test]
    fn read4() {
        assert_eq!(super::read4(U8_BUF).unwrap(), [0xaa; 4]);
    }

    #[test]
    fn read_all() {
        let mut buf = [0; BLOCK_SIZE];

        assert_eq!(
            super::read_all(&U8_BUF[..HALF_BLOCK_SIZE], &mut buf).unwrap(),
            HALF_BLOCK_SIZE
        );
        assert_eq!(&buf[..HALF_BLOCK_SIZE], &U8_BUF[..HALF_BLOCK_SIZE]);
        assert_eq!(&buf[HALF_BLOCK_SIZE..], &[0; HALF_BLOCK_SIZE][..]);

        assert_eq!(super::read_all(U8_BUF, &mut buf).unwrap(), BLOCK_SIZE);
        assert_eq!(&buf[..], U8_BUF);
    }

    #[test]
    fn is_sparse() {
        assert!(super::is_sparse(U32_BUF));

        let buf: Vec<_> = (0..U32_BLOCK_SIZE as u32).collect();
        assert!(!super::is_sparse(&buf));
    }
}
