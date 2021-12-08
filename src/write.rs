//! Sparse image writing and decoding to raw images.

use crate::{
    block::Block,
    ext::{Tell, WriteBlock},
    headers::{ChunkHeader, ChunkType, FileHeader},
    result::Result,
};
use byteorder::{LittleEndian, WriteBytesExt};
use crc::crc32::{self, Hasher32};
use std::io::{prelude::*, BufWriter, SeekFrom};

/// Writes sparse blocks to a sparse image.
pub struct Writer<W: Write + Seek> {
    dst: BufWriter<W>,
    current_chunk: Option<ChunkHeader>,
    current_fill: Option<[u8; 4]>,
    num_blocks: u32,
    num_chunks: u32,
    crc: Option<crc32::Digest>,
    finished: bool,
}

impl<W: Write + Seek> Writer<W> {
    /// Creates a new writer that writes to `w`.
    ///
    /// The created writer skips checksum calculation in favor of
    /// speed. To get a writer that does checksum calculation, use
    /// `Writer::with_crc` instead.
    pub fn new(w: W) -> Result<Self> {
        let mut dst = BufWriter::new(w);
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
            crc: None,
            finished: false,
        })
    }

    /// Creates a new writer that writes to `w` and adds a checksum
    /// at the end.
    pub fn with_crc(w: W) -> Result<Self> {
        let mut writer = Self::new(w)?;
        writer.crc = Some(crc32::Digest::new(crc32::IEEE));
        Ok(writer)
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
            Block::Fill(value) => {
                if self.current_fill.is_none() {
                    self.dst.write_all(value)?;
                    self.current_fill = Some(*value);
                }
            }
            Block::Skip => (),
            Block::Crc32(checksum) => {
                self.dst.write_u32::<LittleEndian>(*checksum)?;
                // CRC chunk size must remain 0, so drop out here already.
                return Ok(());
            }
        }

        if let Some(digest) = self.crc.as_mut() {
            digest.write_block(block);
        }

        chunk.chunk_size += 1;
        Ok(())
    }

    /// Finishes writing the sparse image and flushes any buffered data.
    ///
    /// Consumes the reader as using it afterward would be invalid.
    pub fn close(mut self) -> Result<()> {
        self.finish()
    }

    fn finish(&mut self) -> Result<()> {
        assert!(!self.finished);
        self.finished = true;

        self.write_checksum()?;
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
            (ChunkType::Raw, Block::Raw(_)) | (ChunkType::DontCare, Block::Skip) => true,
            (ChunkType::Fill, Block::Fill(value)) => self.current_fill.unwrap() == *value,
            _ => false,
        }
    }

    fn start_chunk(&mut self, block: &Block) -> Result<()> {
        assert!(self.current_chunk.is_none());

        let (chunk_type, init_size) = match block {
            Block::Raw(_) => (ChunkType::Raw, 0),
            Block::Fill(_) => (ChunkType::Fill, 4),
            Block::Skip => (ChunkType::DontCare, 0),
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

        let pos = self.dst.tell()?;
        let header_off = i64::from(chunk.total_size);
        self.dst.seek(SeekFrom::Current(-header_off))?;
        chunk.write_to(&mut self.dst)?;
        self.dst.seek(SeekFrom::Start(pos))?;

        self.current_fill = None;
        self.num_chunks += 1;
        self.num_blocks += chunk.chunk_size;

        Ok(())
    }

    fn write_checksum(&mut self) -> Result<()> {
        let checksum = match self.crc.as_ref() {
            Some(digest) => digest.sum32(),
            None => return Ok(()),
        };

        let block = Block::Crc32(checksum);
        self.write_block(&block)
    }
}

impl<W: Write + Seek> Drop for Writer<W> {
    fn drop(&mut self) {
        if !self.finished {
            self.finish().ok();
        }
    }
}

/// Decodes sparse blocks and writes them to a raw image.
pub struct Decoder<W: Write + Seek> {
    dst: BufWriter<W>,
    finished: bool,
}

impl<W: Write + Seek> Decoder<W> {
    /// Creates a new decoder that writes to `w`.
    pub fn new(w: W) -> Result<Self> {
        let dst = BufWriter::new(w);
        Ok(Self {
            dst,
            finished: false,
        })
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
            Block::Skip => {
                let offset = i64::from(Block::SIZE) - 1;
                self.dst.seek(SeekFrom::Current(offset))?;
                self.dst.write_all(&[0])?;
            }
            Block::Crc32(_) => (),
        }
        Ok(())
    }

    /// Finishes writing the raw image and flushes any buffered data.
    ///
    /// Consumes the reader as using it afterward would be invalid.
    pub fn close(mut self) -> Result<()> {
        self.finish()
    }

    fn finish(&mut self) -> Result<()> {
        assert!(!self.finished);
        self.finished = true;

        let writer = self.dst.get_mut();
        writer.flush()?;

        Ok(())
    }
}

impl<W: Write + Seek> Drop for Decoder<W> {
    fn drop(&mut self) {
        if !self.finished {
            self.finish().ok();
        }
    }
}
