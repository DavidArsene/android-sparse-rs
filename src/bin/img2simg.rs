extern crate android_sparse as sparse;
#[macro_use]
extern crate clap;

use std::fs::File;
use std::process;

use clap::{App, Arg, ArgMatches};

fn parse_args<'a>() -> ArgMatches<'a> {
    App::new("img2simg")
        .about("Encode a raw file to a sparse file")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(Arg::with_name("raw_file").required(true))
        .arg(Arg::with_name("sparse_file").required(true))
        .arg(
            Arg::with_name("crc")
                .help("Add a checksum to the sparse image")
                .short("c")
                .long("crc"),
        )
        .get_matches()
}

fn img2simg(args: &ArgMatches) -> sparse::Result<()> {
    let fi = File::open(&args.value_of("raw_file").unwrap())?;
    let fo = File::create(&args.value_of("sparse_file").unwrap())?;

    let encoder = sparse::Encoder::new(fi)?;

    let mut writer = if args.is_present("crc") {
        sparse::Writer::with_crc(fo)?
    } else {
        sparse::Writer::new(fo)?
    };

    for block in encoder {
        writer.write_block(&block?)?;
    }
    writer.close()
}

fn main() {
    let args = parse_args();
    img2simg(&args).unwrap_or_else(|err| {
        eprintln!("error: {}", err);
        process::exit(1);
    });
}
