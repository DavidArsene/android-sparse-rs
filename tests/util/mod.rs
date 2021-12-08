#![allow(dead_code)]

use std::{
    fs::{self, File},
    path::PathBuf,
};

use sparse::Block;

pub fn data_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
        .join(name)
}

pub fn data_file(name: &str) -> File {
    let path = data_path(name);
    File::open(path).unwrap()
}

pub fn data(name: &str) -> Vec<u8> {
    let path = data_path(name);
    fs::read(path).unwrap()
}

pub fn test_blocks() -> Vec<Block> {
    let mut raw1 = [0; Block::SIZE as usize];
    for i in 0..raw1.len() {
        raw1[i] = i as u8;
    }
    let mut raw2 = [0; Block::SIZE as usize];
    raw2[1] = 0x66;

    vec![
        Block::Raw(Box::new(raw1)),
        Block::Fill([0xaa; 4]),
        Block::Skip,
        Block::Skip,
        Block::Raw(Box::new(raw2)),
    ]
}
