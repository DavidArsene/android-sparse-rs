extern crate android_sparse as sparse;
#[macro_use]
extern crate clap;

use std::fs::File;
use std::io::{Seek, SeekFrom};

use clap::{App, Arg, ArgMatches};

use sparse::constants::{BLOCK_SIZE, CHUNK_HEADER_SIZE, FILE_HEADER_SIZE};
use sparse::result::Result;

#[derive(Copy, Clone, PartialEq)]
enum Verbosity {
    Normal,
    Verbose,
}

fn parse_args<'a>() -> ArgMatches<'a> {
    App::new("simg_dump")
        .about("Display sparse file info")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(Arg::with_name("sparse_file").required(true))
        .arg(
            Arg::with_name("verbose")
                .help("Verbose output")
                .short("v")
                .long("verbose"),
        )
        .get_matches()
}

fn simg_dump(args: &ArgMatches) -> Result<()> {
    let mut fi = File::open(&args.value_of("sparse_file").unwrap())?;
    let mut sparse_file = sparse::File::new();

    let read_result = sparse::Reader::new(fi.try_clone()?).read(&mut sparse_file);
    // Even if there was an error during parsing, we may still have useful
    // information, so we dump it anyway.
    let verbosity = if args.is_present("verbose") {
        Verbosity::Verbose
    } else {
        Verbosity::Normal
    };
    dump(&sparse_file, verbosity);
    read_result?;

    let spf_end = fi.seek(SeekFrom::Current(0))?;
    let file_end = fi.seek(SeekFrom::End(0))?;
    if spf_end != file_end {
        println!(
            "There are {} bytes of extra data at the end of the file.",
            file_end - spf_end
        );
    }

    Ok(())
}

fn dump(spf: &sparse::File, verbosity: Verbosity) {
    dump_summary(spf);
    if verbosity == Verbosity::Verbose {
        dump_chunks(spf);
    }
}

fn dump_summary(spf: &sparse::File) {
    println!(
        "Total of {} {}-byte output blocks in {} input chunks.",
        spf.num_blocks(),
        BLOCK_SIZE,
        spf.num_chunks()
    );
}

fn dump_chunks(spf: &sparse::File) {
    println!();
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

    println!();
}

fn chunk_type_str(chunk: &sparse::file::Chunk) -> String {
    use sparse::file::Chunk::*;

    match chunk {
        Raw { .. } => "raw".into(),
        Fill { fill, .. } => format!(
            "fill: \\x{:>02x}\\x{:>02x}\\x{:>02x}\\x{:>02x}",
            fill[0], fill[1], fill[2], fill[3]
        ),
        DontCare { .. } => "dont_care".into(),
        Crc32 { crc } => format!("crc32: 0x{:>08x}", crc),
    }
}

fn main() {
    let args = parse_args();
    simg_dump(&args).unwrap_or_else(|err| eprintln!("error: {}", err));
}
