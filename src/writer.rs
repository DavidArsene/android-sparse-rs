use std::io::Write;

use byteorder::{LittleEndian, WriteBytesExt};

use file::{Chunk, File};
use result::Result;

#[derive(Clone, Debug)]
pub struct Writer<W> {
    w: W,
}

impl<W: Write> Writer<W> {
    pub fn new(w: W) -> Self {
        Self { w }
    }

    pub fn write(mut self, sparse_file: &File) -> Result<()> {
        let header = sparse_file.header();
        header.serialize(&mut self.w)?;

        for chunk in sparse_file.chunk_iter() {
            self.write_chunk(chunk, sparse_file)?;
        }

        Ok(())
    }

    fn write_chunk(&mut self, chunk: &Chunk, sparse_file: &File) -> Result<()> {
        let header = sparse_file.chunk_header(chunk);
        header.serialize(&mut self.w)?;

        match *chunk {
            Chunk::Raw { ref buf } => self.w.write_all(buf)?,
            Chunk::Fill { ref fill, .. } => self.w.write_all(fill)?,
            Chunk::DontCare { .. } => {}
            Chunk::Crc32 { crc } => self.w.write_u32::<LittleEndian>(crc)?,
        }

        Ok(())
    }
}
