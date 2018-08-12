extern crate android_sparse as sparse;
#[macro_use]
extern crate clap;

use std::fs::File;

use clap::{App, Arg, ArgMatches};

fn parse_args<'a>() -> ArgMatches<'a> {
    App::new("simg2img")
        .about("Decode a sparse file to a raw file")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(Arg::with_name("sparse_file").required(true))
        .arg(Arg::with_name("raw_file").required(true))
        .arg(
            Arg::with_name("crc")
                .help("Verify all checksums in the sparse image")
                .short("c")
                .long("crc"),
        )
        .get_matches()
}

fn simg2img(args: &ArgMatches) -> sparse::Result<()> {
    let fi = File::open(&args.value_of("sparse_file").unwrap())?;
    let fo = File::create(&args.value_of("raw_file").unwrap())?;

    let reader = if args.is_present("crc") {
        sparse::Reader::with_crc(fi)?
    } else {
        sparse::Reader::new(fi)?
    };

    let mut decoder = sparse::Decoder::new(fo)?;

    for block in reader {
        decoder.write_block(&block?)?;
    }
    decoder.finish()
}

fn main() {
    let args = parse_args();
    simg2img(&args).unwrap_or_else(|err| eprintln!("error: {}", err));
}
