//! Error handling with the `Result` type.

use std::{fmt, io, result};

/// Error type used for all errors produced by this crate.
#[derive(Debug)]
pub enum Error {
    /// Error performing an I/O operation.
    Io(io::Error),
    /// Error during sparse file parsing.
    Parse(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Io(err) => err.fmt(f),
            Error::Parse(s) => f.write_str(s),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

/// Result type used for error handling in this crate.
pub type Result<T> = result::Result<T, Error>;
