extern crate android_sparse as sparse;
extern crate tempfile;

mod util;

use std::io::{prelude::*, SeekFrom};

use sparse::{Decoder, Writer};
use util::{data, read_file, test_blocks};

#[test]
fn write_sparse() {
    let blocks = test_blocks();
    let mut tmpfile = tempfile::tempfile().unwrap();

    let file = tmpfile.try_clone().unwrap();
    let mut writer = Writer::new(file).unwrap();
    for block in &blocks {
        writer.write_block(block).unwrap();
    }
    writer.finish().unwrap();

    tmpfile.seek(SeekFrom::Start(0)).unwrap();
    assert_eq!(read_file(&mut tmpfile), data("hello.simg"));
}

#[test]
fn write_sparse_crc() {
    let blocks = test_blocks();
    let mut tmpfile = tempfile::tempfile().unwrap();

    let file = tmpfile.try_clone().unwrap();
    let mut writer = Writer::with_crc(file).unwrap();
    for block in &blocks {
        writer.write_block(block).unwrap();
    }
    writer.finish().unwrap();

    tmpfile.seek(SeekFrom::Start(0)).unwrap();
    assert_eq!(read_file(&mut tmpfile), data("crc.simg"));
}

#[test]
fn decode_to_raw() {
    let blocks = test_blocks();
    let mut tmpfile = tempfile::tempfile().unwrap();

    let file = tmpfile.try_clone().unwrap();
    let mut decoder = Decoder::new(file).unwrap();
    for block in &blocks {
        decoder.write_block(block).unwrap();
    }
    decoder.finish().unwrap();

    tmpfile.seek(SeekFrom::Start(0)).unwrap();
    assert_eq!(read_file(&mut tmpfile), data("decoded.img"));
}
