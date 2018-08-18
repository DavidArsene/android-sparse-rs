extern crate android_sparse as sparse;
extern crate assert_cmd;
extern crate tempfile;

mod util;

use std::fs::File;
use std::process::Command;

use assert_cmd::prelude::*;

use util::{data, data_path, read_file};

#[test]
fn img2simg() {
    let src = data_path("hello.img");
    let tmpdir = tempfile::tempdir().unwrap();
    let dst = tmpdir.path().join("hello.simg");

    Command::cargo_bin("img2simg")
        .unwrap()
        .arg(&src)
        .arg(&dst)
        .assert()
        .success();

    let mut file = File::open(&dst).unwrap();
    assert_eq!(read_file(&mut file), data("hello.simg"));
}

#[test]
fn img2simg_crc() {
    let src = data_path("hello.img");
    let tmpdir = tempfile::tempdir().unwrap();
    let dst = tmpdir.path().join("hello.simg");

    Command::cargo_bin("img2simg")
        .unwrap()
        .arg("--crc")
        .arg(&src)
        .arg(&dst)
        .assert()
        .success();

    let mut file = File::open(&dst).unwrap();
    assert_eq!(read_file(&mut file), data("crc.simg"));
}

#[test]
fn simg2img() {
    let src = data_path("hello.simg");
    let tmpdir = tempfile::tempdir().unwrap();
    let dst = tmpdir.path().join("hello.img");

    Command::cargo_bin("simg2img")
        .unwrap()
        .arg(&src)
        .arg(&dst)
        .assert()
        .success();

    let mut file = File::open(&dst).unwrap();
    assert_eq!(read_file(&mut file), data("decoded.img"));
}

#[test]
fn simg2img_crc() {
    let src = data_path("crc.simg");
    let tmpdir = tempfile::tempdir().unwrap();
    let dst = tmpdir.path().join("hello.img");

    Command::cargo_bin("simg2img")
        .unwrap()
        .arg(&src)
        .arg(&dst)
        .assert()
        .success();

    let mut file = File::open(&dst).unwrap();
    assert_eq!(read_file(&mut file), data("decoded.img"));
}

#[test]
fn simg2img_invalid_crc() {
    let src = data_path("invalid_crc.simg");
    let tmpdir = tempfile::tempdir().unwrap();
    let dst = tmpdir.path().join("hello.img");

    Command::cargo_bin("simg2img")
        .unwrap()
        .arg("--crc")
        .arg(&src)
        .arg(&dst)
        .assert()
        .failure()
        .stderr("error: Checksum does not match\n");
}

#[test]
fn simg2img_concat() {
    let src = data_path("hello.simg");
    let tmpdir = tempfile::tempdir().unwrap();
    let dst = tmpdir.path().join("hello.img");

    Command::cargo_bin("simg2img")
        .unwrap()
        .arg(format!("{},{}", src.display(), src.display()))
        .arg(&dst)
        .assert()
        .success();

    let expected = data("decoded.img");
    let mut file = File::open(&dst).unwrap();
    let result = read_file(&mut file);

    assert_eq!(&result[..result.len() / 2], &expected[..]);
    assert_eq!(&result[result.len() / 2..], &expected[..]);
}
