use std::convert::{TryFrom, TryInto};
use std::error::Error;
use std::io::{Read, Write};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use result::Result;

pub const FILE_HEADER_SIZE: usize = 28;
pub const CHUNK_HEADER_SIZE: usize = 12;

const FILE_MAGIC: u32 = 0xed26ff3a;
const FILE_FORMAT_VERSION: (u16, u16) = (1, 0);

const CHUNK_MAGIC_RAW: u16 = 0xcac1;
const CHUNK_MAGIC_FILL: u16 = 0xcac2;
const CHUNK_MAGIC_DONT_CARE: u16 = 0xcac3;
const CHUNK_MAGIC_CRC32: u16 = 0xcac4;

#[derive(Clone, Debug)]
pub struct FileHeader {
    pub block_size: u32,
    pub total_blocks: u32,
    pub total_chunks: u32,
    pub image_checksum: u32,
}

impl FileHeader {
    pub fn new(block_size: u32, total_blocks: u32, total_chunks: u32, image_checksum: u32) -> Self {
        Self {
            block_size,
            total_blocks,
            total_chunks,
            image_checksum,
        }
    }

    pub fn deserialize<R: Read>(mut r: R) -> Result<Self> {
        let magic = r.read_u32::<LittleEndian>()?;
        if magic != FILE_MAGIC {
            return Err(format!("Invalid file magic: {:x}", magic).into());
        }

        let version = (r.read_u16::<LittleEndian>()?, r.read_u16::<LittleEndian>()?);
        if version != FILE_FORMAT_VERSION {
            return Err(format!("Invalid file format version: {:?}", version).into());
        }

        let file_header_size = r.read_u16::<LittleEndian>()?;
        if file_header_size != FILE_HEADER_SIZE as u16 {
            return Err(format!("Invalid file header size: {}", file_header_size).into());
        }
        let chunk_header_size = r.read_u16::<LittleEndian>()?;
        if chunk_header_size != CHUNK_HEADER_SIZE as u16 {
            return Err(format!("Invalid chunk header size: {}", chunk_header_size).into());
        }

        Ok(Self {
            block_size: r.read_u32::<LittleEndian>()?,
            total_blocks: r.read_u32::<LittleEndian>()?,
            total_chunks: r.read_u32::<LittleEndian>()?,
            image_checksum: r.read_u32::<LittleEndian>()?,
        })
    }

    pub fn serialize<W: Write>(&self, mut w: W) -> Result<()> {
        w.write_u32::<LittleEndian>(FILE_MAGIC)?;

        let (maj_version, min_version) = FILE_FORMAT_VERSION;
        w.write_u16::<LittleEndian>(maj_version)?;
        w.write_u16::<LittleEndian>(min_version)?;

        w.write_u16::<LittleEndian>(FILE_HEADER_SIZE as u16)?;
        w.write_u16::<LittleEndian>(CHUNK_HEADER_SIZE as u16)?;

        w.write_u32::<LittleEndian>(self.block_size)?;
        w.write_u32::<LittleEndian>(self.total_blocks)?;
        w.write_u32::<LittleEndian>(self.total_chunks)?;
        w.write_u32::<LittleEndian>(self.image_checksum)?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct ChunkHeader {
    pub chunk_type: ChunkType,
    pub chunk_size: u32,
    pub total_size: u32,
}

impl ChunkHeader {
    pub fn deserialize<R: Read>(mut r: R) -> Result<Self> {
        let chunk_type = r.read_u16::<LittleEndian>()?.try_into()?;
        r.read_u16::<LittleEndian>()?; // reserved1

        Ok(Self {
            chunk_type: chunk_type,
            chunk_size: r.read_u32::<LittleEndian>()?,
            total_size: r.read_u32::<LittleEndian>()?,
        })
    }

    pub fn serialize<W: Write>(&self, mut w: W) -> Result<()> {
        w.write_u16::<LittleEndian>(self.chunk_type.into())?;
        w.write_u16::<LittleEndian>(0)?; // reserved1

        w.write_u32::<LittleEndian>(self.chunk_size)?;
        w.write_u32::<LittleEndian>(self.total_size)?;

        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ChunkType {
    Raw,
    Fill,
    DontCare,
    Crc32,
}

impl From<ChunkType> for u16 {
    fn from(original: ChunkType) -> u16 {
        match original {
            ChunkType::Raw => CHUNK_MAGIC_RAW,
            ChunkType::Fill => CHUNK_MAGIC_FILL,
            ChunkType::DontCare => CHUNK_MAGIC_DONT_CARE,
            ChunkType::Crc32 => CHUNK_MAGIC_CRC32,
        }
    }
}

impl TryFrom<u16> for ChunkType {
    type Error = Box<Error>;
    fn try_from(original: u16) -> Result<Self> {
        match original {
            CHUNK_MAGIC_RAW => Ok(ChunkType::Raw),
            CHUNK_MAGIC_FILL => Ok(ChunkType::Fill),
            CHUNK_MAGIC_DONT_CARE => Ok(ChunkType::DontCare),
            CHUNK_MAGIC_CRC32 => Ok(ChunkType::Crc32),
            n => Err(format!("Invalid chunk magic: {}", n).into()),
        }
    }
}
