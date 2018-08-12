# android-sparse

An implementation of Android's sparse file format in Rust.

Sparse is a simple file compression format. It is supported by Android's
fastboot protocol as a means of dealing with images that are too large to fit
into the device's memory at once. Android images are often provided in sparse
format and must be decoded before they can be further inspected.

There is no official documentation of the sparse format. However,
[libsparse](https://android.googlesource.com/platform/system/core/+/master/libsparse/)
is part of the Android Open Source Project (AOSP). It provides utilities to
convert from raw to sparse images and back, called `img2simg` and `simg2img`,
respectively. Unfortunately, these tools are not part of the Android SDK. To
use them, one has to download the AOSP and build them from source, which can
be a rather time-consuming undertaking.

This project reimplements parts of libsparse in Rust to make working with
sparse images less of a hassle. While android-sparse doesn't implement all
features of libsparse, it supports the main use cases of:

* converting raw to sparse images (`img2simg`)
* converting sparse to raw images (`simg2img`)

Additionally, being implemented in Rust it has a couple of advantages over
libsparse, namely guaranteed memory safety and a significantly simpler build
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

Encoding a raw image to a sparse image:

    $ img2simg <raw_image> <sparse_image>

The `-c`/`--crc` flag makes `img2simg` write a checksum to the sparse image:

    $ img2simg --crc <raw_image> <sparse_image>

### Decoding

Decoding a sparse image to a raw image:

    $ simg2img <sparse_image> <raw_image>

The `-c`/`--crc` flag makes `simg2img` check the checksums included in the
sparse image. Decoding is aborted if they don't match.

    $ simg2img --crc <sparse_image> <raw_image>

## License

This project is licensed under the MIT license ([LICENSE](LICENSE) or
http://opensource.org/licenses/MIT).

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in android-sparse by you, shall be licensed as above, without any
additional terms or conditions.
