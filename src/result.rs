//! Error handling with this crate's `Result` type.

use std::error::Error;
use std::result::Result as StdResult;

/// Specialized `Result` type used for error handling in this crate.
pub type Result<T> = StdResult<T, Box<Error>>;
