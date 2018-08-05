extern crate android_sparse as sparse;
#[macro_use]
extern crate clap;

use std::fs::File;
use std::io::BufWriter;

use clap::{App, Arg, ArgMatches};

use sparse::result::Result;

fn parse_args<'a>() -> ArgMatches<'a> {
    App::new("simg2img")
        .about("Decode a sparse file to a raw file")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(Arg::with_name("sparse_file").required(true))
        .arg(Arg::with_name("raw_file").required(true))
        .arg(
            Arg::with_name("crc")
                .help("Check the sparse image checksum")
                .short("c")
                .long("crc"),
        )
        .get_matches()
}

fn simg2img(args: &ArgMatches) -> Result<()> {
    let fi = File::open(&args.value_of("sparse_file").unwrap())?;
    let fo = File::create(&args.value_of("raw_file").unwrap())?;

    let mut sparse_file = sparse::File::new();
    let mut reader = sparse::Reader::new(fi);
    if args.is_present("crc") {
        reader = reader.with_crc();
    }
    reader.read(&mut sparse_file)?;

    let writer = BufWriter::new(fo);
    sparse::Decoder::new(writer).write(&sparse_file)?;

    Ok(())
}

fn main() {
    let args = parse_args();
    simg2img(&args).unwrap_or_else(|err| eprintln!("error: {}", err));
}
