extern crate android_sparse as sparse;

use std::env;
use std::fs::File;

use sparse::constants::{BLOCK_SIZE, CHUNK_HEADER_SIZE, FILE_HEADER_SIZE};
use sparse::result::Result;

struct Args {
    src: String,
}

impl Args {
    fn parse() -> Result<Self> {
        let args = env::args().skip(1).collect::<Vec<String>>();
        if args.len() != 1 {
            println!("usage: simg_dump <sparse_image_file>");
            return Err("Invalid number of arguments".into());
        }
        Ok(Self {
            src: args[0].clone(),
        })
    }
}

fn simg_dump(args: &Args) -> Result<()> {
    let fi = File::open(&args.src)?;

    let mut sparse_file = sparse::File::new();
    sparse::Reader::new(fi).read(&mut sparse_file)?;

    dump(&sparse_file);

    Ok(())
}

fn dump(spf: &sparse::File) {
    dump_summary(spf);
    dump_chunks(spf);
}


fn dump_summary(spf: &sparse::File) {
    println!(
        "Total of {} {}-byte output blocks in {} input chunks.",
        spf.num_blocks(),
        BLOCK_SIZE,
        spf.num_chunks()
    );

    if spf.checksum() != 0 {
        println!("checksum=0x{:>08x}", spf.checksum());
    }
}


fn dump_chunks(spf: &sparse::File) {
    println!("");
    println!("       |       input_bytes       |   output_blocks   |");
    println!(" chunk |   offset   |   number   | offset  |  number | type");
    println!("-----------------------------------------------------------------");

    let mut bytes_off = u64::from(FILE_HEADER_SIZE + CHUNK_HEADER_SIZE);
    let mut blocks_off = 0_u64;
    for (i, chunk) in spf.chunk_iter().enumerate() {
        let chunk_num = i + 1;
        let sparse_size = chunk.sparse_size() - u32::from(CHUNK_HEADER_SIZE);
        let num_blocks = chunk.num_blocks();

        println!(
            " {:>5} | {:>10} | {:>10} | {:>7} | {:>7} | {}",
            chunk_num,
            bytes_off,
            sparse_size,
            blocks_off,
            num_blocks,
            chunk_type_str(chunk),
        );

        bytes_off += u64::from(sparse_size) + u64::from(CHUNK_HEADER_SIZE);
        blocks_off += u64::from(num_blocks);
    }
}

fn chunk_type_str(chunk: &sparse::file::Chunk) -> String {
    use sparse::file::Chunk::*;

    match *chunk {
        Raw { .. } => "raw".into(),
        Fill { fill, .. } => format!(
            "fill: \\x{:>02x}\\x{:>02x}\\x{:>02x}\\x{:>02x}",
            fill[0],
            fill[1],
            fill[2],
            fill[3]
        ),
        DontCare { .. } => "dont_care".into(),
        Crc32 { crc } => format!("crc32: 0x{:>08x}", crc),
    }
}

fn main() {
    Args::parse()
        .and_then(|args| simg_dump(&args))
        .unwrap_or_else(|err| eprintln!("error: {}", err));
}
