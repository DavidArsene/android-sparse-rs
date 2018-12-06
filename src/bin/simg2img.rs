extern crate android_sparse as sparse;

use clap::{crate_authors, crate_version, App, Arg, ArgMatches};
use std::{fs::File, process};

fn parse_args<'a>() -> ArgMatches<'a> {
    App::new("simg2img")
        .about("Decode one or more sparse images to a raw image")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(
            Arg::with_name("sparse_images")
                .help("Paths of the sparse images, separated by commas")
                .required(true)
                .multiple(true)
                .require_delimiter(true),
        )
        .arg(
            Arg::with_name("raw_image")
                .help("Path of the output raw image")
                .required(true),
        )
        .arg(
            Arg::with_name("crc")
                .help("Verify all checksums in the sparse image")
                .short("c")
                .long("crc"),
        )
        .get_matches()
}

fn simg2img(args: &ArgMatches) -> sparse::Result<()> {
    let mut fis = Vec::new();
    for path in args.values_of("sparse_images").unwrap() {
        fis.push(File::open(path)?);
    }
    let fo = File::create(&args.value_of("raw_image").unwrap())?;

    let mut decoder = sparse::Decoder::new(fo)?;

    for fi in fis {
        let reader = if args.is_present("crc") {
            sparse::Reader::with_crc(fi)?
        } else {
            sparse::Reader::new(fi)?
        };

        for block in reader {
            decoder.write_block(&block?)?;
        }
    }

    decoder.close()
}

fn main() {
    let args = parse_args();
    simg2img(&args).unwrap_or_else(|err| {
        eprintln!("error: {}", err);
        process::exit(1);
    });
}
