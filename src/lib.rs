//! Library to parse JSON
#![deny(missing_docs)]
//#![no_std]
#![forbid(unsafe_code)]
#[macro_use]
extern crate alloc;

mod error;
mod lexer;
mod parser;
pub use error::*;
pub use parser::{parse, Object, StringValue, Value};
