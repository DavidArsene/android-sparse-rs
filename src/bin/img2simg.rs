extern crate android_sparse as sparse;

use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::{File, OpenOptions};

/// Encode a raw image to a sparse image
#[derive(Parser)]
#[command(name = "img2simg", version, arg_required_else_help(true))]
struct Args {
    /// Add checksum to output image
    #[clap(short, long)]
    crc: bool,

    /// Overwrite output image
    #[clap(short, long)]
    force: bool,

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
    let size = fi.metadata()?.len();

    let encoder = sparse::Encoder::new(fi)?;

    let fo = OpenOptions::new().write(true).create(true)
        .create_new(!args.force)
        .open(args.sparse_image)?;

    let mut writer = sparse::Writer::new(fo, args.crc)?;

    let bar = ProgressBar::new(size);
    let template = "{elapsed} {bar:80} {bytes} / {total_bytes}";
    bar.set_style(ProgressStyle::with_template(template)?.progress_chars("█▉▊▋▌▍▎▏  "));

    for block in encoder {
        writer.write_block(&block?)?;
        bar.inc(sparse::block::Block::SIZE as u64);
    }

    bar.finish();
    writer.close()
}
