//! Provides UCS2 string types for usage in AVM1 and AVM2.
//!
//! Internally, these types are represeted by a sequence of 1-byte or 2-bytes (wide) code units,
//! that may contains null bytes or unpaired surrogates.
//!
//! To match Flash behavior, the string length is limited to 2³¹-1 code units;
//! any attempt to create a longer string will panic.

#[macro_use]
mod common;

mod avm;
mod buf;
mod ops;
mod parse;
mod pattern;
mod ptr;
mod tables;
pub mod utils;

#[cfg(test)]
mod tests;

pub use ptr::{WStr, MAX_STRING_LEN};

pub use avm::AvmString;
pub use buf::WString;
pub use common::Units;
pub use ops::{CharIndices, Chars, Iter, Split, WStrToUtf8};
pub use parse::{FromWStr, Integer};
pub use pattern::Pattern;

use std::borrow::Borrow;

use common::panic_on_invalid_length;

/// Flattens a slice of strings, placing `sep` as a separator between each.
#[inline]
pub fn join<E: Borrow<WStr>, S: Borrow<WStr>>(elems: &[E], sep: &S) -> WString {
    crate::string::ops::str_join(elems, sep.borrow())
}
