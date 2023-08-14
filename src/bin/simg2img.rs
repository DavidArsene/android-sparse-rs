extern crate android_sparse as sparse;

use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::{File, OpenOptions};

/// Decode one or more sparse images to a raw image
#[derive(Parser)]
#[command(name = "simg2img", version, arg_required_else_help(true))]
struct Args {
    /// Verify checksums
    #[clap(short, long)]
    crc: bool,

    /// Overwrite output image
    #[clap(short, long)]
    force: bool,

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

    let fo = OpenOptions::new().write(true).create(true)
        .create_new(!args.force)
        .open(args.raw_image)?;

    let mut decoder = sparse::Decoder::new(fo)?;

    for fi in fis {
        let reader = sparse::Reader::new(fi, args.crc)?;

        let bar = ProgressBar::new(reader.size as u64);
        let template = "{elapsed} {bar:80} {bytes} / {total_bytes}";
        bar.set_style(ProgressStyle::with_template(template)?.progress_chars("█▉▊▋▌▍▎▏  "));

        for block in reader {
            decoder.write_block(&block?)?;
            bar.inc(sparse::block::Block::SIZE as u64);
        }

        bar.finish();
    }

    decoder.close()
}
