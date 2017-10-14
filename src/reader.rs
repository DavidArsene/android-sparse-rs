use std::io::Read;

use file::{Chunk, File};
use headers::{ChunkHeader, ChunkType, FileHeader};
use result::Result;

#[derive(Clone, Debug)]
pub struct Reader<R> {
    r: R,
    block_size: Option<u32>,
}

impl<R: Read> Reader<R> {
    pub fn new(r: R) -> Self {
        Self {
            r: r,
            block_size: None,
        }
    }

    pub fn read(mut self) -> Result<File> {
        let header = FileHeader::deserialize(&mut self.r)?;
        self.block_size = Some(header.block_size);
        let mut sparse_file = File::new(header.block_size);

        for _ in 0..header.total_chunks {
            self.read_chunk(&mut sparse_file)?;
        }

        Ok(sparse_file)
    }

    fn read_chunk(&mut self, sparse_file: &mut File) -> Result<()> {
        let header = ChunkHeader::deserialize(&mut self.r)?;
        let block_size = self.block_size.unwrap();
        let size = header.chunk_size * block_size;

        let chunk = match header.chunk_type {
            ChunkType::Raw => {
                let mut buf = vec![0; size as usize];
                self.r.read_exact(&mut buf)?;
                Chunk::Raw { buf }
            }
            ChunkType::Fill => {
                let mut fill = [0; 4];
                self.r.read_exact(&mut fill)?;
                Chunk::Fill { fill, size }
            }
            ChunkType::DontCare => Chunk::DontCare { size },
            ChunkType::Crc32 => {
                let mut crc = [0; 4];
                self.r.read_exact(&mut crc)?;
                Chunk::Crc32 { crc }
            }
        };

        sparse_file.add_chunk(chunk)?;
        Ok(())
    }
}
