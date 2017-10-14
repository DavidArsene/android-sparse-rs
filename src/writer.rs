use std::io::Write;

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
            self.write_chunk(&chunk, &sparse_file)?;
        }

        Ok(())
    }

    fn write_chunk(&mut self, chunk: &Chunk, sparse_file: &File) -> Result<()> {
        let header = sparse_file.chunk_header(chunk);
        header.serialize(&mut self.w)?;

        let body = match *chunk {
            Chunk::Raw { ref buf } => &buf[..],
            Chunk::Fill { ref fill, .. } => fill,
            Chunk::DontCare { .. } => &[],
            Chunk::Crc32 { ref crc } => crc,
        };
        self.w.write(body)?;

        Ok(())
    }
}
