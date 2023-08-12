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

pub mod block;
pub mod read;
pub mod write;

mod ext;
mod headers;

pub use self::{
    block::Block,
    read::{Encoder, Reader},
    write::{Decoder, Writer},
};
