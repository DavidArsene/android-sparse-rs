extern crate android_sparse as sparse;

mod util;

use crate::util::{data, test_blocks};
use sparse::{Decoder, Writer};
use std::{
    fs::File,
    io::{prelude::*, SeekFrom},
};

fn read_from_start(file: &mut File) -> Vec<u8> {
    let mut result = Vec::new();
    file.seek(SeekFrom::Start(0)).unwrap();
    file.read_to_end(&mut result).unwrap();
    result
}

#[test]
fn write_sparse() {
    let blocks = test_blocks();
    let mut tmpfile = tempfile::tempfile().unwrap();

    let file = tmpfile.try_clone().unwrap();
    let mut writer = Writer::new(file, false).unwrap();
    for block in &blocks {
        writer.write_block(block).unwrap();
    }
    writer.close().unwrap();

    assert_eq!(read_from_start(&mut tmpfile), data("hello.simg"));
}

#[test]
fn write_sparse_crc() {
    let blocks = test_blocks();
    let mut tmpfile = tempfile::tempfile().unwrap();

    let file = tmpfile.try_clone().unwrap();
    let mut writer = Writer::new(file, true).unwrap();
    for block in &blocks {
        writer.write_block(block).unwrap();
    }
    writer.close().unwrap();

    assert_eq!(read_from_start(&mut tmpfile), data("crc.simg"));
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
    decoder.close().unwrap();

    assert_eq!(read_from_start(&mut tmpfile), data("decoded.img"));
}
