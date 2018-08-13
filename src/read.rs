//! Sparse image reading and encoding from raw images.

use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, ErrorKind};

use byteorder::{LittleEndian, ReadBytesExt};
use crc::crc32;
use crc::crc32::Hasher32;

use block::Block;
use ext::WriteBlock;
use headers::{ChunkHeader, ChunkType, FileHeader};
use result::{Error, Result};

/// Reads sparse blocks from a sparse image.
///
/// Implements the `Iterator` trait, so sparse blocks can be read from
/// a reader by iterating over it.
pub struct Reader {
    src: BufReader<File>,
    current_chunk: Option<ChunkHeader>,
    current_fill: Option<[u8; 4]>,
    remaining_chunks: u32,
    crc: Option<crc32::Digest>,
    finished: bool,
}

impl Reader {
    /// Creates a new reader that reads from `file`.
    ///
    /// The created reader skips checksum verification in favor of
    /// speed. To get a reader that does checksum verification, use
    /// `Reader::with_crc` instead.
    pub fn new(file: File) -> Result<Self> {
        let mut src = BufReader::new(file);
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

    /// Creates a new reader that reads from `file` and verifies all
    /// included checksums.
    pub fn with_crc(file: File) -> Result<Self> {
        let mut reader = Self::new(file)?;
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
                let mut buf = [0; Block::SIZE as usize];
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

impl Iterator for Reader {
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

/// Reads blocks from a raw image and encodes them into sparse blocks.
///
/// Implements the `Iterator` trait, so sparse blocks can be read from
/// an encoder by iterating over it.
pub struct Encoder {
    src: File,
    finished: bool,
}

impl Encoder {
    /// Creates a new encoder that reads from `file`.
    pub fn new(file: File) -> Result<Self> {
        Ok(Self {
            src: file,
            finished: false,
        })
    }

    fn read_block(&self) -> Result<Option<Block>> {
        let mut buf = [0; Block::SIZE as usize];
        let bytes_read = read_all(&self.src, &mut buf)?;

        let block = match bytes_read {
            0 => None,
            _ => Some(self.encode_block(buf)),
        };
        Ok(block)
    }

    fn encode_block(&self, buf: [u8; Block::SIZE as usize]) -> Block {
        if is_sparse(&buf) {
            let value = read4(&buf[..]).unwrap();
            if value == [0; 4] {
                Block::Skip
            } else {
                Block::Fill(value)
            }
        } else {
            Block::Raw(Box::new(buf))
        }
    }
}

impl Iterator for Encoder {
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
                buf = &mut tmp[n..]
            }
            Err(ref e) if e.kind() == ErrorKind::Interrupted => (),
            Err(e) => return Err(e.into()),
        };
    }

    Ok(buf_size - buf.len())
}

fn is_sparse(buf: &[u8]) -> bool {
    let mut parts = buf.chunks(4);
    let first = parts.next().unwrap();
    parts.all(|p| p == first)
}
