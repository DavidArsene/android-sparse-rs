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
        .author("Jan Teske <jan.teske@gmail.com>")
        .arg(Arg::with_name("sparse_file").required(true))
        .arg(Arg::with_name("raw_file").required(true))
        .get_matches()
}

fn simg2img(args: &ArgMatches) -> Result<()> {
    let fi = File::open(&args.value_of("sparse_file").unwrap())?;
    let fo = File::create(&args.value_of("raw_file").unwrap())?;

    let writer = BufWriter::new(fo);

    let mut sparse_file = sparse::File::new();
    sparse::Reader::new(fi).with_crc().read(&mut sparse_file)?;
    sparse::Decoder::new(writer).write(&sparse_file)?;

    Ok(())
}

fn main() {
    let args = parse_args();
    simg2img(&args).unwrap_or_else(|err| eprintln!("error: {}", err));
}
