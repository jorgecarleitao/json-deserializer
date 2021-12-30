//! Library to parse JSON
#![deny(missing_docs)]
#[forbid(unsafe_code)]
mod error;
mod lexer;
mod parser;
pub use error::*;
pub use parser::{parse, Value};
