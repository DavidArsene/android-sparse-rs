// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at http://rust-lang.org/COPYRIGHT.
// Modifications copyright 2017 Jan Teske.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! `TryFrom` and `TryInto` traits for attempted conversions between types.
//! Copied from the standard library to remove the dependency on nightly Rust.

use std::error::Error;
use std::fmt;

/// An attempted conversion that consumes `self`, which may or may not be
/// expensive.
///
/// Library authors should not directly implement this trait, but should prefer
/// implementing the [`TryFrom`] trait, which offers greater flexibility and
/// provides an equivalent `TryInto` implementation for free, thanks to a
/// blanket implementation in the standard library. For more information on this,
/// see the documentation for [`Into`].
///
/// [`TryFrom`]: trait.TryFrom.html
/// [`Into`]: trait.Into.html
pub trait TryInto<T>: Sized {
    /// The type returned in the event of a conversion error.
    type Error;

    /// Performs the conversion.
    fn try_into(self) -> Result<T, Self::Error>;
}

/// Attempt to construct `Self` via a conversion.
pub trait TryFrom<T>: Sized {
    /// The type returned in the event of a conversion error.
    type Error;

    /// Performs the conversion.
    fn try_from(value: T) -> Result<Self, Self::Error>;
}

// TryFrom implies TryInto
impl<T, U> TryInto<U> for T
where
    U: TryFrom<T>,
{
    type Error = U::Error;

    fn try_into(self) -> Result<U, U::Error> {
        U::try_from(self)
    }
}

// TryFrom impls for integral types

/// The error type returned when a checked integral type conversion fails.
#[derive(Debug, Copy, Clone)]
pub struct TryFromIntError(());

impl TryFromIntError {
    pub fn __description(&self) -> &str {
        "out of range integral type conversion attempted"
    }
}

impl Error for TryFromIntError {
    fn description(&self) -> &str {
        self.__description()
    }
}

impl fmt::Display for TryFromIntError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.__description().fmt(fmt)
    }
}

impl TryFrom<usize> for u32 {
    type Error = TryFromIntError;

    fn try_from(u: usize) -> Result<Self, TryFromIntError> {
        if u > (<Self>::max_value() as usize) {
            Err(TryFromIntError(()))
        } else {
            Ok(u as Self)
        }
    }
}
