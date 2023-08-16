extern crate android_sparse as sparse;

mod util;

use self::util::{data, data_path};
use assert_cmd::prelude::*;
use std::{fs, process::Command};

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
        .stderr("Error: Checksum does not match\n");
}
