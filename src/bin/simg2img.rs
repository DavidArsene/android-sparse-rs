extern crate android_sparse as sparse;

use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use std::{fs::{File, OpenOptions}, io::Read};

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
    #[clap(last = true)]
    raw_image: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // If no output image is specified, use the input image
    // as the output and read from stdin.
    let fis: Vec<Box<dyn Read>> = if args.raw_image.is_some() {
        let mut fis: Vec<Box<dyn Read>> = Vec::new();
        for sparse_image in &args.sparse_images {
            fis.push(Box::new(File::open(sparse_image)?));
        }
        fis
    } else {
        vec![Box::new(std::io::stdin())]
    };

    let dst_stdin = args.sparse_images.first().unwrap().into();
    let dst = args.raw_image.unwrap_or(dst_stdin);

    let fo = OpenOptions::new().write(true).create(true)
        .create_new(!args.force).open(dst)?;

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
