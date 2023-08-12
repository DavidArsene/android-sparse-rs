extern crate android_sparse as sparse;

use clap::Parser;
use std::fs::File;

/// Decode one or more sparse images to a raw image
#[derive(Parser)]
#[command(name = "simg2img", version, arg_required_else_help(true))]
struct Args {
    /// Verify checksums
    #[clap(short, long)]
    crc: bool,

    /// Input sparse images
    #[arg(required = true)]
    sparse_images: Vec<String>,

    /// Output raw image
    #[clap()]
    raw_image: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let fis = args.sparse_images.iter()
        .map(File::open).collect::<Result<Vec<_>, _>>()?;

    let output = File::create(args.raw_image)?;
    let mut decoder = sparse::Decoder::new(output)?;

    for fi in fis {
        let reader = sparse::Reader::new(fi, args.crc)?;

        for block in reader {
            decoder.write_block(&block?)?;
        }
    }

    decoder.close()
}
