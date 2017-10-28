use std::fs::File as StdFile;
use std::slice::Iter;

use convert::TryInto;
use headers::ChunkType;
use headers::CHUNK_HEADER_SIZE;
use result::Result;

pub type ChunkIter<'a> = Iter<'a, Chunk>;

#[derive(Debug)]
pub struct File {
    block_size: u32,
    backing_file: Option<StdFile>,
    chunks: Vec<Chunk>,
}

impl File {
    pub fn new(block_size: u32) -> Self {
        Self {
            block_size: block_size,
            backing_file: None,
            chunks: Vec::new(),
        }
    }

    pub fn set_backing_file(&mut self, file: StdFile) {
        self.backing_file = Some(file);
    }

    pub fn block_size(&self) -> u32 {
        self.block_size
    }

    pub fn checksum(&self) -> u32 {
        // TODO
        0
    }

    pub fn num_blocks(&self) -> u32 {
        self.chunks
            .iter()
            .fold(0, |sum, chunk| sum + chunk.raw_size() / self.block_size)
    }

    pub fn num_chunks(&self) -> u32 {
        self.chunks
            .len()
            .try_into()
            .expect("number of chunks doesn't fit into u32")
    }

    pub fn add_raw(&mut self, buf: &[u8]) -> Result<()> {
        if buf.len() % self.block_size as usize != 0 {
            return Err("bytes size must be multiple of block_size".into());
        }

        let new_buf = buf;
        if let Some(&mut Chunk::Raw { ref mut buf }) = self.chunks.iter_mut().last() {
            buf.extend(new_buf.iter().cloned());
            return Ok(());
        }

        let buf = new_buf.to_vec();
        self.chunks.push(Chunk::Raw { buf });
        Ok(())
    }

    pub fn add_raw_file_backed(&mut self, offset: u64, size: u32) -> Result<()> {
        let backing_file = match self.backing_file {
            Some(ref f) => f,
            None => return Err("Sparse File not created with backing file".into()),
        };
        if size % self.block_size != 0 {
            return Err("size must be multiple of block_size".into());
        }

        let (new_offset, new_size) = (offset, size);
        if let Some(&mut Chunk::RawFileBacked {
            offset,
            ref mut size,
            ..
        }) = self.chunks.iter_mut().last()
        {
            if new_offset == offset + u64::from(*size) {
                *size += new_size;
                return Ok(());
            }
        }

        self.chunks.push(Chunk::RawFileBacked {
            file: backing_file.try_clone()?,
            offset: offset,
            size: size,
        });
        Ok(())
    }

    pub fn add_fill(&mut self, fill: [u8; 4], size: u32) -> Result<()> {
        if size % self.block_size != 0 {
            return Err("size must be multiple of block_size".into());
        }

        let (new_fill, new_size) = (fill, size);
        if let Some(&mut Chunk::Fill { fill, ref mut size }) = self.chunks.iter_mut().last() {
            if fill == new_fill {
                *size += new_size;
                return Ok(());
            }
        }

        self.chunks.push(Chunk::Fill { fill, size });
        Ok(())
    }

    pub fn add_dont_care(&mut self, size: u32) -> Result<()> {
        if size % self.block_size != 0 {
            return Err("size must be multiple of block_size".into());
        }

        let new_size = size;
        if let Some(&mut Chunk::DontCare { ref mut size }) = self.chunks.iter_mut().last() {
            *size += new_size;
            return Ok(());
        }

        self.chunks.push(Chunk::DontCare { size });
        Ok(())
    }

    pub fn add_crc32(&mut self, crc: u32) -> Result<()> {
        self.chunks.push(Chunk::Crc32 { crc });
        Ok(())
    }

    pub fn chunk_iter(&self) -> ChunkIter {
        self.chunks.iter()
    }
}

#[derive(Debug)]
pub enum Chunk {
    Raw { buf: Vec<u8> },
    RawFileBacked {
        file: StdFile,
        offset: u64,
        size: u32,
    },
    Fill { fill: [u8; 4], size: u32 },
    DontCare { size: u32 },
    Crc32 { crc: u32 },
}

impl Chunk {
    pub fn size(&self) -> u32 {
        let body_size = match *self {
            Chunk::Raw { ref buf } => buf.len()
                .try_into()
                .expect("chunk size doesn't fit into u32"),
            Chunk::RawFileBacked { size, .. } => size,
            Chunk::Fill { .. } | Chunk::Crc32 { .. } => 4,
            Chunk::DontCare { .. } => 0,
        };
        u32::from(CHUNK_HEADER_SIZE) + body_size
    }

    pub fn raw_size(&self) -> u32 {
        match *self {
            Chunk::Raw { ref buf } => buf.len()
                .try_into()
                .expect("raw chunk size doesn't fit into u32"),
            Chunk::RawFileBacked { size, .. } |
            Chunk::Fill { size, .. } |
            Chunk::DontCare { size } => size,
            Chunk::Crc32 { .. } => 0,
        }
    }

    pub fn chunk_type(&self) -> ChunkType {
        match *self {
            Chunk::Raw { .. } | Chunk::RawFileBacked { .. } => ChunkType::Raw,
            Chunk::Fill { .. } => ChunkType::Fill,
            Chunk::DontCare { .. } => ChunkType::DontCare,
            Chunk::Crc32 { .. } => ChunkType::Crc32,
        }
    }
}
