//! An implementation of Android's sparse file format.
//!
//! Enables reading and writing sparse images, as well as encoding from and
//! decoding to raw images.
//!
//! For consistency, this documentation refers to actual files on the file
//! system as *images*. A *raw image* is an image in its original,
//! uncompressed form. A *sparse image* is a sparse-encoded image.
//! `android-sparse` implements means to convert raw to sparse images and vice
//! versa, via an intermediate representation referred to as *sparse file*:
//!
//! ```text
//!  -------                --------               --------
//! | raw   | --Encoder--> | sparse | <--Reader-- | sparse |
//! | image | <--Decoder-- | file   | --Writer--> | image  |
//!  -------                --------               --------
//! ```

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
