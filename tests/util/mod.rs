#![allow(dead_code)]

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use sparse::Block;

pub fn data_file(name: &str) -> File {
    let path = Path::new(file!())
        .ancestors()
        .nth(2)
        .unwrap()
        .join("data")
        .join(name);
    File::open(path).unwrap()
}

pub fn data(name: &str) -> Vec<u8> {
    let mut file = data_file(name);
    let mut data = Vec::new();
    file.read_to_end(&mut data).unwrap();
    data
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
