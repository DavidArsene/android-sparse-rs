use std::io::Write;

use byteorder::{LittleEndian, WriteBytesExt};

use file::{Chunk, File};
use result::Result;

#[derive(Debug)]
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

#[derive(Debug)]
pub struct Decoder<W> {
    w: W,
}

impl<W: Write> Decoder<W> {
    pub fn new(w: W) -> Self {
        Self { w }
    }

    pub fn write(mut self, sparse_file: &File) -> Result<()> {
        for chunk in sparse_file.chunk_iter() {
            self.write_chunk(chunk)?;
        }

        Ok(())
    }

    fn write_chunk(&mut self, chunk: &Chunk) -> Result<()> {
        match *chunk {
            Chunk::Raw { ref buf } => self.w.write_all(buf)?,
            Chunk::Fill { ref fill, size } => for i in 0..size {
                let idx = i as usize % 4;
                self.w.write_all(&fill[idx..idx + 1])?;
            },
            Chunk::DontCare { size } => for _ in 0..size {
                self.w.write_all(&[0])?;
            },
            Chunk::Crc32 { .. } => (),
        };

        Ok(())
    }
}
