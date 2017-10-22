use std::cmp;
use std::fs::File as StdFile;
use std::io::{ErrorKind, Read, Seek, SeekFrom, Write};

use byteorder::{LittleEndian, WriteBytesExt};

use file::{Chunk, File};
use result::Result;

const COPY_BUF_SIZE: usize = 4096;

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
            Chunk::RawFileBacked {
                ref file,
                offset,
                size,
            } => copy_from_file(file, &mut self.w, offset, size as usize)?,
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
            Chunk::RawFileBacked {
                ref file,
                offset,
                size,
            } => copy_from_file(file, &mut self.w, offset, size as usize)?,
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

fn copy_from_file<W: Write>(
    file: &StdFile,
    writer: &mut W,
    offset: u64,
    size: usize,
) -> Result<()> {
    let mut file = file.try_clone()?;
    file.seek(SeekFrom::Start(offset))?;
    copy_exact(&mut file, writer, size)
}

fn copy_exact<R: Read, W: Write>(reader: &mut R, writer: &mut W, size: usize) -> Result<()> {
    let mut buf = [0; COPY_BUF_SIZE];

    let mut written = 0;
    while written != size {
        let read_size = cmp::min(size - written, COPY_BUF_SIZE);
        if let Err(err) = reader.read_exact(&mut buf[..read_size]) {
            if err.kind() == ErrorKind::Interrupted {
                continue;
            }
            return Err(err.into());
        }
        writer.write_all(&buf[..read_size])?;
        written += read_size;
    }

    Ok(())
}