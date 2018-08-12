//! Sparse image writing and decoding to raw images.

use std::fs::File;
use std::io::prelude::*;
use std::io::{BufWriter, SeekFrom};

use byteorder::{LittleEndian, WriteBytesExt};

use block::Block;
use headers::{ChunkHeader, ChunkType, FileHeader};
use result::Result;

/// Writes sparse blocks to a sparse image.
pub struct Writer {
    dst: BufWriter<File>,
    current_chunk: Option<ChunkHeader>,
    current_fill: Option<[u8; 4]>,
    num_blocks: u32,
    num_chunks: u32,
}

impl Writer {
    /// Creates a new writer that writes to `file`.
    pub fn new(file: File) -> Result<Self> {
        let mut dst = BufWriter::new(file);
        // We cannot write the file header until we know the total number of
        // blocks and chunks. So we skip it here and write it at the end in
        // `finish`.
        dst.seek(SeekFrom::Current(i64::from(FileHeader::SIZE)))?;

        Ok(Self {
            dst,
            current_chunk: None,
            current_fill: None,
            num_blocks: 0,
            num_chunks: 0,
        })
    }

    /// Writes a sparse block to this writer.
    ///
    /// The sparse block is converted into the sparse file format and
    /// written to this decoder's destination.
    pub fn write_block(&mut self, block: &Block) -> Result<()> {
        if !self.can_merge(block) {
            self.finish_chunk()?;
            self.start_chunk(block)?;
        }

        let chunk = self.current_chunk.as_mut().unwrap();

        match block {
            Block::Raw(buf) => {
                self.dst.write_all(&**buf)?;
                chunk.total_size += Block::SIZE;
            }
            Block::Fill(value) => if self.current_fill.is_none() {
                self.dst.write_all(value)?;
                self.current_fill = Some(*value);
            },
            Block::DontCare => (),
            Block::Crc32(checksum) => self.dst.write_u32::<LittleEndian>(*checksum)?,
        }

        chunk.chunk_size += 1;
        Ok(())
    }

    /// Finish writing the sparse image and flush any buffered data.
    pub fn finish(mut self) -> Result<()> {
        self.finish_chunk()?;

        // Like libsparse, we always set the checksum value in the file header
        // to 0. If checksum writing is enabled, we append a Crc32 chunk at
        // the end of the file instead.
        let image_checksum = 0;
        let header = FileHeader {
            total_blocks: self.num_blocks,
            total_chunks: self.num_chunks,
            image_checksum,
        };

        self.dst.seek(SeekFrom::Start(0))?;
        header.write_to(&mut self.dst)?;

        self.dst.flush()?;
        Ok(())
    }

    fn can_merge(&self, block: &Block) -> bool {
        let chunk = match self.current_chunk.as_ref() {
            Some(c) => c,
            None => return false,
        };

        match (chunk.chunk_type, block) {
            (ChunkType::Raw, Block::Raw(_)) | (ChunkType::DontCare, Block::DontCare) => true,
            (ChunkType::Fill, Block::Fill(value)) => self.current_fill.unwrap() == *value,
            _ => false,
        }
    }

    fn start_chunk(&mut self, block: &Block) -> Result<()> {
        assert!(self.current_chunk.is_none());

        let (chunk_type, init_size) = match block {
            Block::Raw(_) => (ChunkType::Raw, 0),
            Block::Fill(_) => (ChunkType::Fill, 4),
            Block::DontCare => (ChunkType::DontCare, 0),
            Block::Crc32(_) => (ChunkType::Crc32, 4),
        };

        let chunk = ChunkHeader {
            chunk_type,
            chunk_size: 0,
            total_size: init_size + u32::from(ChunkHeader::SIZE),
        };

        // We cannot write the chunk header until we know the total number of
        // blocks in the chunk. So we skip it here and write it later in
        // `finish_chunk`.
        let header_size = i64::from(ChunkHeader::SIZE);
        self.dst.seek(SeekFrom::Current(header_size))?;

        self.current_chunk = Some(chunk);

        Ok(())
    }

    fn finish_chunk(&mut self) -> Result<()> {
        let chunk = match self.current_chunk.take() {
            Some(c) => c,
            None => return Ok(()),
        };

        let pos = self.dst.seek(SeekFrom::Current(0))?;
        let header_off = i64::from(chunk.total_size);
        self.dst.seek(SeekFrom::Current(-header_off))?;
        chunk.write_to(&mut self.dst)?;
        self.dst.seek(SeekFrom::Start(pos))?;

        self.current_fill = None;
        self.num_chunks += 1;
        self.num_blocks += chunk.chunk_size;

        Ok(())
    }
}

/// Decodes sparse blocks and writes them to a raw image.
pub struct Decoder {
    dst: BufWriter<File>,
}

impl Decoder {
    /// Creates a new decoder that writes to `file`.
    pub fn new(file: File) -> Result<Self> {
        let dst = BufWriter::new(file);
        Ok(Self { dst })
    }

    /// Writes a sparse block to this decoder.
    ///
    /// The sparse block is decoded into its raw form and written to
    /// this decoder's destination.
    pub fn write_block(&mut self, block: &Block) -> Result<()> {
        match block {
            Block::Raw(buf) => self.dst.write_all(&**buf)?,
            Block::Fill(value) => {
                let count = Block::SIZE as usize / 4;
                for _ in 0..count {
                    self.dst.write_all(value)?;
                }
            }
            Block::DontCare => {
                let buf = [0; Block::SIZE as usize];
                self.dst.write_all(&buf)?;
            }
            Block::Crc32(_) => (),
        }
        Ok(())
    }

    /// Finish writing the raw image and flush any buffered data.
    pub fn finish(mut self) -> Result<()> {
        self.dst.flush()?;
        Ok(())
    }
}
