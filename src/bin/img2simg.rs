extern crate android_sparse as sparse;

use std::env;
use std::fs::File;
use std::io::BufWriter;

use sparse::result::Result;

struct Args {
    src: String,
    dst: String,
}

impl Args {
    fn parse() -> Result<Self> {
        let args = env::args().skip(1).collect::<Vec<String>>();
        if args.len() != 2 {
            println!("usage: img2simg <raw_image_file> <sparse_image_file>");
            return Err("Invalid number of arguments".into());
        }
        Ok(Self {
            src: args[0].clone(),
            dst: args[1].clone(),
        })
    }
}

fn img2simg(args: &Args) -> Result<()> {
    let fi = File::open(&args.src)?;
    let fo = File::create(&args.dst)?;

    let writer = BufWriter::new(fo);

    let mut sparse_file = sparse::File::new();
    sparse::Encoder::new(&mut sparse_file).read_from(fi)?;
    sparse::Writer::new(&mut sparse_file).write_to(writer)?;

    Ok(())
}

fn main() {
    Args::parse()
        .and_then(|args| img2simg(&args))
        .unwrap_or_else(|err| eprintln!("error: {}", err));
}
