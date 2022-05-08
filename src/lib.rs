//! Library to parse JSON
#![deny(missing_docs)]
//#![no_std]
#![forbid(unsafe_code)]
#[macro_use]
extern crate alloc;

mod boolean;
mod error;
mod lexer;
mod null;
mod number;
mod parser;
mod string;
pub use error::*;
pub use parser::{parse, Object, Value};
pub use string::StringValue;
