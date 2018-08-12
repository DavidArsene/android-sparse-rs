//! An implementation of Android's sparse file format.
//!
//! Enables reading and writing sparse images, as well as encoding from and
//! decoding to raw images:
//!
//! ```text
//!  --------               --------                -------
//! | sparse | --Reader--> | sparse | --Decoder--> | raw   |
//! | image  | <--Writer-- | blocks | <--Encoder-- | image |
//!  --------               --------                -------
//! ```

#![deny(missing_docs)]

extern crate byteorder;
extern crate crc;

pub mod block;
pub mod read;
pub mod result;
pub mod write;

mod headers;

pub use block::Block;
pub use read::{Encoder, Reader};
pub use result::Result;
pub use write::{Decoder, Writer};
