# android-sparse

An implementation of Android's sparse file format in Rust.

Sparse is a simple file compression format. It is supported by Android's
fastboot protocol as a means of dealing with images that are too large to fit
into the device's memory at once. Android images are often provided in sparse
format and must be decoded before they can be further inspected.

There is no official documentation of the sparse format. However,
[libsparse](https://android.googlesource.com/platform/system/core/+/master/libsparse/)
is part of the Android Open Source Project (AOSP). It provides utilities to
convert from raw to sparse files and back, called `img2simg` and `simg2img`,
respectively. Unfortunately, these tools are not part of the Android SDK. To
use them, one has to download the AOSP and build them from source, which can be
a rather time-consuming undertaking.

This project reimplements parts of libsparse in Rust to make working with
sparse files less of a hassle. While android-sparse doesn't implement all
features of libsparse, it supports the main use cases of:

* converting raw to sparse files (`img2simg`)
* converting sparse to raw files (`simg2img`)
* inspecting sparse files (`simg_dump`)

Additionally, being implemented in Rust it has a couple of advantages over
libsparse, namely guaranteed memory safety and a significanty simpler build
process across all major operating systems.

## Installation

To build android-sparse, you need a working installation of Rust. Check out
https://www.rustup.rs for instructions.

The latest stable version of android-sparse is available on
[crates.io](https://crates.io/crates/android-sparse). You can install it with
`cargo`:

    $ cargo install android-sparse

This installs the android-sparse tools in `cargo`'s bin directory. If you
installed Rust via `rustup`, this directory is located at:

* `$HOME/.cargo/bin` on Unix
* `%USERPROFILE%\.cargo\bin` on Windows

## Usage

### Encoding

Encoding a raw file to a sparse file:

    $ img2simg <raw_file> <sparse_file>

The `-c`/`--crc` flag makes `img2simg` write an image checksum to the sparse file:

    $ img2simg --crc <raw_file> <sparse_file>

### Decoding

Decoding a sparse file to a raw file:

    $ simg2img <sparse_file> <raw_file>

The `-c`/`--crc` flag makes `simg2img` check the checksums included in the sparse
file. Decoding is aborted if they don't match.

    $ simg2img --crc <sparse_file> <raw_file>

### Inspection

Displaying sparse file info:

    $ simg_dump <sparse_file>

By default, only a summary is printed. To also print information about each
chunk contained in the sparse file, use the `-v`/`--verbose` flag:

    $ simg_dump -v <sparse_file>

## License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in android-sparse by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.
