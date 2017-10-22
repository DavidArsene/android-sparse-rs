use std::fs::File as StdFile;
use std::io::{ErrorKind, Read, Seek, SeekFrom};
use std::io::Result as IoResult;

use byteorder::{LittleEndian, ReadBytesExt};

use file::File;
use headers::{ChunkHeader, ChunkType, FileHeader};
use result::Result;

#[derive(Debug)]
enum Source<R> {
    Reader(R),
    File(StdFile),
}

impl<R: Read> Read for Source<R> {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        match *self {
            Source::Reader(ref mut r) => r.read(buf),
            Source::File(ref mut f) => f.read(buf),
        }
    }
}

#[derive(Debug)]
pub struct Reader<R> {
    source: Source<R>,
    block_size: Option<u32>,
}

impl<R: Read> Reader<R> {
    pub fn from_reader(r: R) -> Self {
        Self {
            source: Source::Reader(r),
            block_size: None,
        }
    }
}

impl Reader<StdFile> {
    pub fn from_file(file: StdFile) -> Self {
        Self {
            source: Source::File(file),
            block_size: None,
        }
    }
}

impl<R: Read> Reader<R> {
    pub fn read(mut self) -> Result<File> {
        let header = FileHeader::deserialize(&mut self.source)?;
        self.block_size = Some(header.block_size);

        let mut sparse_file = match self.source {
            Source::Reader(_) => File::new(header.block_size),
            Source::File(ref f) => File::with_backing_file(f.try_clone()?, header.block_size),
        };

        for _ in 0..header.total_chunks {
            self.read_chunk(&mut sparse_file)?;
        }

        Ok(sparse_file)
    }

    fn read_chunk(&mut self, sparse_file: &mut File) -> Result<()> {
        let header = ChunkHeader::deserialize(&mut self.source)?;
        let block_size = self.block_size.expect("block_size not set");
        let size = header.chunk_size * block_size;

        match header.chunk_type {
            ChunkType::Raw => {
                let mut buf = vec![0; size as usize];
                self.source.read_exact(&mut buf)?;
                sparse_file.add_raw(&buf)?;
            }
            ChunkType::Fill => {
                let mut fill = [0; 4];
                self.source.read_exact(&mut fill)?;
                sparse_file.add_fill(fill, size)?;
            }
            ChunkType::DontCare => sparse_file.add_dont_care(size)?,
            ChunkType::Crc32 => {
                let crc = self.source.read_u32::<LittleEndian>()?;
                sparse_file.add_crc32(crc)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct Encoder<R> {
    source: Source<R>,
    block_size: u32,
}

impl<R: Read> Encoder<R> {
    pub fn from_reader(r: R, block_size: u32) -> Self {
        Self {
            source: Source::Reader(r),
            block_size: block_size,
        }
    }
}

impl Encoder<StdFile> {
    pub fn from_file(file: StdFile, block_size: u32) -> Self {
        Self {
            source: Source::File(file),
            block_size: block_size,
        }
    }
}

impl<R: Read> Encoder<R> {
    pub fn read(mut self) -> Result<File> {
        let mut sparse_file = match self.source {
            Source::Reader(_) => File::new(self.block_size),
            Source::File(ref f) => File::with_backing_file(f.try_clone()?, self.block_size),
        };

        let block_size = self.block_size as usize;
        let mut block = vec![0; block_size];
        loop {
            let bytes_read = read_all(&mut self.source, &mut block)?;
            self.read_block(&block[..bytes_read], &mut sparse_file)?;
            if bytes_read != block_size {
                break;
            }
        }

        Ok(sparse_file)
    }

    fn read_block(&self, block: &[u8], sparse_file: &mut File) -> Result<()> {
        if block.is_empty() {
            return Ok(());
        }

        if self.is_sparse_block(block) {
            let mut fill = [0; 4];
            fill.copy_from_slice(&block[..4]);
            if fill == [0; 4] {
                sparse_file.add_dont_care(self.block_size)?;
            } else {
                sparse_file.add_fill(fill, self.block_size)?;
            }
            return Ok(());
        }

        match self.source {
            Source::Reader(_) => sparse_file.add_raw(block)?,
            Source::File(ref f) => {
                let mut file = f.try_clone()?;
                let curr_offset = file.seek(SeekFrom::Current(0))?;
                let offset = curr_offset - self.block_size as u64;
                sparse_file.add_raw_file_backed(offset, self.block_size)?;
            }
        }

        Ok(())
    }

    fn is_sparse_block(&self, block: &[u8]) -> bool {
        if block.len() != self.block_size as usize {
            return false;
        }

        let mut words = block.chunks(4);
        let first = words.next().expect("block is empty");
        for word in words {
            if word != first {
                return false;
            }
        }

        true
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
