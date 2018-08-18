extern crate android_sparse as sparse;
extern crate tempfile;

mod util;

use std::io::{prelude::*, SeekFrom};

use sparse::{Decoder, Writer};
use util::{data, test_blocks};

#[test]
fn write_sparse() {
    let blocks = test_blocks();
    let mut tmpfile = tempfile::tempfile().unwrap();
    let expected = data("hello.simg");

    let file = tmpfile.try_clone().unwrap();
    let mut writer = Writer::new(file).unwrap();
    for block in &blocks {
        writer.write_block(block).unwrap();
    }
    writer.finish().unwrap();

    let mut result = Vec::new();
    tmpfile.seek(SeekFrom::Start(0)).unwrap();
    tmpfile.read_to_end(&mut result).unwrap();

    assert_eq!(&result, &expected);
}

#[test]
fn write_sparse_crc() {
    let blocks = test_blocks();
    let mut tmpfile = tempfile::tempfile().unwrap();
    let expected = data("crc.simg");

    let file = tmpfile.try_clone().unwrap();
    let mut writer = Writer::with_crc(file).unwrap();
    for block in &blocks {
        writer.write_block(block).unwrap();
    }
    writer.finish().unwrap();

    let mut result = Vec::new();
    tmpfile.seek(SeekFrom::Start(0)).unwrap();
    tmpfile.read_to_end(&mut result).unwrap();

    assert_eq!(&result, &expected);
}

#[test]
fn decode_to_raw() {
    let blocks = test_blocks();
    let mut tmpfile = tempfile::tempfile().unwrap();
    let expected = data("decoded.img");

    let file = tmpfile.try_clone().unwrap();
    let mut decoder = Decoder::new(file).unwrap();
    for block in &blocks {
        decoder.write_block(block).unwrap();
    }
    decoder.finish().unwrap();

    let mut result = Vec::new();
    tmpfile.seek(SeekFrom::Start(0)).unwrap();
    tmpfile.read_to_end(&mut result).unwrap();

    assert_eq!(&result, &expected);
}
