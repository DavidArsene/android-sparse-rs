extern crate android_sparse as sparse;
#[macro_use]
extern crate clap;

use std::fs::File;
use std::io::BufWriter;

use clap::{App, Arg, ArgMatches};

use sparse::result::Result;

fn parse_args<'a>() -> ArgMatches<'a> {
    App::new("img2simg")
        .about("Encode a raw file to a sparse file")
        .version(crate_version!())
        .author("Jan Teske <jan.teske@gmail.com>")
        .arg(Arg::with_name("raw_file").required(true))
        .arg(Arg::with_name("sparse_file").required(true))
        .arg(
            Arg::with_name("crc")
                .help("Compute the sparse image checksum")
                .short("c")
                .long("crc"),
        )
        .get_matches()
}

fn img2simg(args: &ArgMatches) -> Result<()> {
    let fi = File::open(&args.value_of("raw_file").unwrap())?;
    let fo = File::create(&args.value_of("sparse_file").unwrap())?;

    let writer = BufWriter::new(fo);

    let mut sparse_file = sparse::File::new();
    let encoder = sparse::Encoder::new(fi);

    let mut writer = sparse::Writer::new(writer);
    if args.is_present("crc") {
        writer = writer.with_crc();
    }

    encoder.read(&mut sparse_file)?;
    writer.write(&sparse_file)?;

    Ok(())
}

fn main() {
    let args = parse_args();
    img2simg(&args).unwrap_or_else(|err| eprintln!("error: {}", err));
}
