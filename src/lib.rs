#![feature(try_from)]

extern crate byteorder;

pub mod file;
pub mod result;

mod headers;
mod read;
mod write;

pub use read::{Encoder, Reader};
pub use write::{Decoder, Writer};
