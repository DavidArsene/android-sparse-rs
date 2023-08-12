extern crate android_sparse as sparse;

use clap::Parser;
use std::fs::File;

/// Encode a raw image to a sparse image
#[derive(Parser)]
#[command(name = "img2simg", version, arg_required_else_help(true))]
struct Args {
    /// Add checksum to output image
    #[clap(short, long)]
    crc: bool,

    /// Input raw image
    #[arg()]
    raw_image: String,

    /// Output sparse image
    #[clap()]
    sparse_image: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    
    let fi = File::open(args.raw_image)?;
    let fo = File::create(args.sparse_image)?;

    let encoder = sparse::Encoder::new(fi)?;

    let mut writer = sparse::Writer::new(fo, args.crc)?;

    for block in encoder {
        writer.write_block(&block?)?;
    }
    writer.close()
}
