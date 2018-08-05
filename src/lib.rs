//! An implementation of Android's sparse file format.
//!
//! Enables reading and writing sparse files, as well as encoding from and
//! decoding to raw images.

#![warn(missing_docs)]

extern crate byteorder;
extern crate crc;

pub mod constants;
pub mod file;
pub mod read;
pub mod result;
pub mod write;

mod convert;
mod headers;

pub use file::File;
pub use read::{Encoder, Reader};
pub use write::{Decoder, Writer};
