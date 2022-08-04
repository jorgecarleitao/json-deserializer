use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;

use super::array::parse_array;
use super::boolean::parse_false;
use super::boolean::parse_true;
use super::error::*;
use super::null::parse_null;
use super::number::parse_number;
use super::object::parse_object;
use super::string::parse_string;

/// Typedef for the inside of an object.
#[cfg(not(feature = "preserve_order"))]
pub type Object<'a> = alloc::collections::BTreeMap<String, Value<'a>>;
/// Typedef for the inside of an object.
#[cfg(feature = "preserve_order")]
pub type Object<'a> = indexmap::IndexMap<String, Value<'a>>;

/// Reference to JSON data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Number<'a> {
    /// A float (contains exactly 1 period)
    Float(&'a [u8], &'a [u8]),
    /// An integer (contains exactly 0 periods)
    Integer(&'a [u8], &'a [u8]),
}

/// Reference to JSON data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value<'a> {
    /// A `null`
    Null,
    /// A string (i.e. something quoted; quotes are not part of this. Data has not been UTF-8 validated)
    String(Cow<'a, str>),
    /// A number (i.e. something starting with a number with an optional period)
    Number(Number<'a>),
    /// A bool (i.e. `false` or `true`)
    Bool(bool),
    /// An object (i.e. items inside curly brackets `{}` separated by colon `:` and comma `,`)
    Object(Object<'a>),
    /// An array (i.e. items inside squared brackets `[]` separated by comma `,`)
    Array(Vec<Value<'a>>),
}

/// Parses JSON-compliant bytes into [`Value`]
/// # Errors
/// If and only if `json` is not valid JSON.
/// # Panics
/// If and only if there is not enough memory to allocate.
pub fn parse(mut json: &[u8]) -> Result<Value, Error> {
    parse_value(&mut json)
}

pub fn parse_value<'b, 'a>(values: &'b mut &'a [u8]) -> Result<Value<'a>, Error> {
    skip_unused(values);
    let token = current_token(values)?;
    match token {
        b'{' => parse_object(values).map(Value::Object),
        b'[' => parse_array(values).map(Value::Array),
        b'"' => parse_string(values).map(Value::String),
        b'n' => parse_null(values).map(|_| Value::Null),
        b't' => parse_true(values).map(|_| Value::Bool(true)),
        b'f' => parse_false(values).map(|_| Value::Bool(false)),
        b'0'..=b'9' | b'-' => parse_number(values).map(Value::Number),
        other => Err(Error::InvalidToken(other)),
    }
}

#[inline]
pub fn skip_unused(values: &mut &[u8]) {
    while let [first, rest @ ..] = values {
        if !matches!(first, b'\n' | b' ' | b'\r' | b'\t') {
            break;
        }
        *values = rest;
    }
}

#[inline]
pub fn current_token(values: &[u8]) -> Result<u8, Error> {
    if let Some(t) = values.get(0) {
        Ok(*t)
    } else {
        Err(Error::InvalidEOF)
    }
}
