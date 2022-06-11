//! Library to parse JSON
#![deny(missing_docs)]
#![no_std]
#![forbid(unsafe_code)]
#[macro_use]
extern crate alloc;

mod array;
mod boolean;
mod error;
mod null;
mod number;
mod object;
mod parser;
mod string;

pub use error::*;
pub use parser::{parse, Number, Object, Value};
