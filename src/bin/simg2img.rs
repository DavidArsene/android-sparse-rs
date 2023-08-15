extern crate android_sparse as sparse;

use indicatif::{ProgressBar, ProgressStyle};
use std::{fs::{File, OpenOptions}, io::Read};

/// Decode a sparse image to a raw image
#[derive(argh::FromArgs)]
struct Args {
    /// verify checksum
    #[argh(switch, short = 'c')]
    crc: bool,

    /// overwrite output image
    #[argh(switch, short = 'f')]
    force: bool,

    /// input sparse image
    #[argh(positional)]
    sparse_image: String,

    /// output raw image
    #[argh(positional)]
    raw_image: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let args: Args = argh::from_env();

    let (mut stdin_read, mut file_read, dst);

    // If no output image is specified, use the input image
    // as the output and read from stdin.
    let fi: &mut dyn Read = if args.raw_image.is_some() {

        file_read = File::open(args.sparse_image)?;
        dst = args.raw_image.unwrap();
        &mut file_read
    } else {

        stdin_read = std::io::stdin();
        dst = args.sparse_image;
        &mut stdin_read
    };
    let reader = sparse::Reader::new(fi, args.crc)?;

    let fo = OpenOptions::new().write(true).create(true)
        .create_new(!args.force).open(dst)?;

    let mut decoder = sparse::Decoder::new(fo)?;

    let bar = ProgressBar::new(reader.size as u64);
    let template = "{elapsed} {bar:80} {bytes} / {total_bytes}";
    bar.set_style(ProgressStyle::with_template(template)?.progress_chars("█▉▊▋▌▍▎▏  "));

    for block in reader {
        decoder.write_block(&block?)?;
        bar.inc(sparse::block::Block::SIZE as u64);
    }

    bar.finish();
    decoder.close()
}
