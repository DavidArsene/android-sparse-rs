extern crate android_sparse as sparse;

mod util;

use self::util::{data_file, test_blocks};
use sparse::{Block, Encoder, Reader};

#[test]
fn read_sparse() {
    let file = data_file("hello.simg");
    let expected = test_blocks();

    let reader = Reader::new(file, false).unwrap();
    let blocks: Vec<_> = reader.map(|r| r.unwrap()).collect();
    assert_eq!(blocks.len(), expected.len());

    for (blk, exp) in blocks.iter().zip(expected.iter()) {
        assert_eq!(blk, exp);
    }
}

#[test]
fn read_sparse_with_crc() {
    let file = data_file("crc.simg");
    let mut expected = test_blocks();
    expected.push(Block::Crc32(0xffb880a5));

    let reader = Reader::new(file, true).unwrap();
    let blocks: Vec<_> = reader.map(|r| r.unwrap()).collect();
    assert_eq!(blocks.len(), expected.len());

    for (blk, exp) in blocks.iter().zip(expected.iter()) {
        assert_eq!(blk, exp);
    }
}

#[test]
fn read_sparse_with_invalid_crc() {
    let file = data_file("invalid_crc.simg");

    let mut reader = Reader::new(file, true).unwrap();
    assert!(reader.nth(5).unwrap().is_err());
}

#[test]
fn encode_raw() {
    let file = data_file("hello.img");
    let expected = test_blocks();

    let encoder = Encoder::new(file).unwrap();
    let blocks: Vec<_> = encoder.map(|r| r.unwrap()).collect();
    assert_eq!(blocks.len(), expected.len());

    for (blk, exp) in blocks.iter().zip(expected.iter()) {
        assert_eq!(blk, exp);
    }
}
