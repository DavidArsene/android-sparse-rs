extern crate android_sparse as sparse;
extern crate assert_cmd;
extern crate tempfile;

mod util;

use std::fs;
use std::process::Command;

use assert_cmd::prelude::*;

use util::{data, data_path};

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

    assert_eq!(fs::read(&dst).unwrap(), data("hello.simg"));
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

    assert_eq!(fs::read(&dst).unwrap(), data("crc.simg"));
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

    assert_eq!(fs::read(&dst).unwrap(), data("decoded.img"));
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

    assert_eq!(fs::read(&dst).unwrap(), data("decoded.img"));
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
    let result = fs::read(&dst).unwrap();

    assert_eq!(&result[..result.len() / 2], &expected[..]);
    assert_eq!(&result[result.len() / 2..], &expected[..]);
}
