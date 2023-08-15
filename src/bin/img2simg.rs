extern crate android_sparse as sparse;

use indicatif::{ProgressBar, ProgressStyle};
use std::fs::{File, OpenOptions};

/// Encode a raw image to a sparse image
#[derive(argh::FromArgs)]
struct Args {
    /// add checksum to output image
    #[argh(switch, short = 'c')]
    crc: bool,

    /// overwrite output image
    #[argh(switch, short = 'f')]
    force: bool,

    /// input raw image
    #[argh(positional)]
    raw_image: String,

    /// output sparse image
    #[argh(positional)]
    sparse_image: String,
}

fn main() -> anyhow::Result<()> {
    let args: Args = argh::from_env();
    
    let fi = File::open(args.raw_image)?;
    let size = fi.metadata()?.len();

    let encoder = sparse::Encoder::new(fi)?;

    let fo = OpenOptions::new().write(true).create(true)
        .create_new(!args.force).open(args.sparse_image)?;

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
