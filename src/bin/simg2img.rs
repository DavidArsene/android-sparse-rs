extern crate android_sparse as sparse;

use std::env;
use std::fs::File;
use std::io::{BufReader, BufWriter};

use sparse::result::Result;

struct Args {
    src: String,
    dst: String,
}

impl Args {
    fn parse() -> Result<Self> {
        let args = env::args().skip(1).collect::<Vec<String>>();
        if args.len() != 2 {
            println!("usage: simg2img <sparse_image_file> <raw_image_file>");
            return Err("Invalid number of arguments".into());
        }
        Ok(Self {
            src: args[0].clone(),
            dst: args[1].clone(),
        })
    }
}

fn simg2img(args: &Args) -> Result<()> {
    let fi = File::open(&args.src)?;
    let fo = File::create(&args.dst)?;

    let reader = BufReader::new(fi);
    let writer = BufWriter::new(fo);

    let sparse_file = sparse::Reader::new(reader).read()?;
    sparse::Decoder::new(writer).write(&sparse_file)?;

    Ok(())
}

fn main() {
    Args::parse()
        .and_then(|args| simg2img(&args))
        .unwrap_or_else(|err| eprintln!("error: {}", err));
}
