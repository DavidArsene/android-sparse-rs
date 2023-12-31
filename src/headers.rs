use anyhow::{Result, ensure, bail};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::prelude::*;

use crate::block::Block;

const FILE_MAGIC: u32 = 0xed26_ff3a;
const FILE_FORMAT_VERSION: (u16, u16) = (1, 0);

const CHUNK_MAGIC_RAW: u16 = 0xcac1;
const CHUNK_MAGIC_FILL: u16 = 0xcac2;
const CHUNK_MAGIC_DONT_CARE: u16 = 0xcac3;
const CHUNK_MAGIC_CRC32: u16 = 0xcac4;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FileHeader {
    pub(crate) total_blocks: u32,
    pub(crate) total_chunks: u32,
    pub(crate) image_checksum: u32,
}

impl FileHeader {
    pub(crate) const SIZE: u16 = 28;

    pub(crate) fn read_from<R: Read>(mut r: R) -> Result<Self> {
        let magic = r.read_u32::<LittleEndian>()?;
        ensure!(magic == FILE_MAGIC, "Invalid file magic: {magic:x}");

        let version = (r.read_u16::<LittleEndian>()?, r.read_u16::<LittleEndian>()?);
        ensure!(version == FILE_FORMAT_VERSION, "Invalid file format version: {version:?}");

        let file_header_size = r.read_u16::<LittleEndian>()?;
        ensure!(file_header_size == Self::SIZE, "Invalid file header size: {file_header_size}");

        let chunk_header_size = r.read_u16::<LittleEndian>()?;
        ensure!(chunk_header_size == ChunkHeader::SIZE, "Invalid chunk header size: {chunk_header_size}");

        let block_size = r.read_u32::<LittleEndian>()?;
        ensure!(block_size == Block::SIZE, "Invalid block size: {block_size}");

        Ok(Self {
            total_blocks: r.read_u32::<LittleEndian>()?,
            total_chunks: r.read_u32::<LittleEndian>()?,
            image_checksum: r.read_u32::<LittleEndian>()?,
        })
    }

    /// Writes this sparse file header into `w`.
    pub(crate) fn write_to<W: Write>(&self, mut w: W) -> Result<()> {
        w.write_u32::<LittleEndian>(FILE_MAGIC)?;

        let (maj_version, min_version) = FILE_FORMAT_VERSION;
        w.write_u16::<LittleEndian>(maj_version)?;
        w.write_u16::<LittleEndian>(min_version)?;

        w.write_u16::<LittleEndian>(Self::SIZE)?;
        w.write_u16::<LittleEndian>(ChunkHeader::SIZE)?;
        w.write_u32::<LittleEndian>(Block::SIZE)?;

        w.write_u32::<LittleEndian>(self.total_blocks)?;
        w.write_u32::<LittleEndian>(self.total_chunks)?;
        w.write_u32::<LittleEndian>(self.image_checksum)?;

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u16)]
pub(crate) enum ChunkType {
    Raw = CHUNK_MAGIC_RAW,
    Fill = CHUNK_MAGIC_FILL,
    DontCare = CHUNK_MAGIC_DONT_CARE,
    Crc32 = CHUNK_MAGIC_CRC32,
}

impl ChunkType {
    fn from_magic(magic: u16) -> Result<Self> {
        match magic {
            CHUNK_MAGIC_RAW => Ok(ChunkType::Raw),
            CHUNK_MAGIC_FILL => Ok(ChunkType::Fill),
            CHUNK_MAGIC_DONT_CARE => Ok(ChunkType::DontCare),
            CHUNK_MAGIC_CRC32 => Ok(ChunkType::Crc32),
            _ => bail!("Invalid chunk magic: {magic:x}"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ChunkHeader {
    pub(crate) chunk_type: ChunkType,
    pub(crate) chunk_size: u32,
    pub(crate) total_size: u32,
}

impl ChunkHeader {
    pub(crate) const SIZE: u16 = 12;

    pub(crate) fn read_from<R: Read>(mut r: R) -> Result<Self> {
        let magic = r.read_u16::<LittleEndian>()?;
        let chunk_type = ChunkType::from_magic(magic)?;
        r.read_u16::<LittleEndian>()?; // reserved1

        Ok(Self {
            chunk_type,
            chunk_size: r.read_u32::<LittleEndian>()?,
            total_size: r.read_u32::<LittleEndian>()?,
        })
    }

    pub(crate) fn write_to<W: Write>(&self, mut w: W) -> Result<()> {
        w.write_u16::<LittleEndian>(self.chunk_type as u16)?;
        w.write_u16::<LittleEndian>(0)?; // reserved1

        w.write_u32::<LittleEndian>(self.chunk_size)?;
        w.write_u32::<LittleEndian>(self.total_size)?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const FILE_HEADER_BYTES: &[u8] = &[
        0x3a, 0xff, 0x26, 0xed, 0x01, 0x00, 0x00, 0x00, 0x1c, 0x00, 0x0c, 0x00, 0x00, 0x10, 0x00,
        0x00, 0x00, 0x00, 0x04, 0x00, 0x96, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    const FILE_HEADER: FileHeader = FileHeader {
        total_blocks: 262144,
        total_chunks: 1430,
        image_checksum: 0,
    };

    const CHUNK_HEADER_BYTES: &[u8] = &[
        0xc1, 0xca, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x0c, 0x10, 0x00, 0x00,
    ];

    const CHUNK_HEADER: ChunkHeader = ChunkHeader {
        chunk_type: ChunkType::Raw,
        chunk_size: 1,
        total_size: 4108,
    };

    #[test]
    fn read_file_header() {
        let header = FileHeader::read_from(FILE_HEADER_BYTES).unwrap();
        assert_eq!(header, FILE_HEADER);
    }

    #[test]
    fn write_file_header() {
        let mut bytes = Vec::new();
        FILE_HEADER.write_to(&mut bytes).unwrap();
        assert_eq!(&bytes[..], FILE_HEADER_BYTES);
    }

    #[test]
    fn read_chunk_header() {
        let header = ChunkHeader::read_from(CHUNK_HEADER_BYTES).unwrap();
        assert_eq!(header, CHUNK_HEADER);
    }

    #[test]
    fn write_chunk_header() {
        let mut bytes = Vec::new();
        CHUNK_HEADER.write_to(&mut bytes).unwrap();
        assert_eq!(&bytes[..], CHUNK_HEADER_BYTES);
    }
}
