use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use super::boolean::parse_false as _parse_false;
use super::boolean::parse_true as _parse_true;
use super::error::*;
use super::null::parse_null as _parse_null;
use super::number::parse_number as _parse_number;
use super::string::{parse_string as _parse_string, StringValue};

/// The state of the lexer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum State {
    Finished,
    None,           // something like a white space, that does not contribute
    Null,           // the `null` (n,u,l,l)
    ObjectStart,    // {
    ObjectEnd,      // }
    ColonSeparator, // :
    ItemEnd,        // ,
    ArrayStart,     // [
    ArrayEnd,       // ]
    Bool(bool),     // f,a,l,s,e or t,r,u,e
    String,         // something between double quotes
    Number,         // a number
}

/// The transition state of the lexer
#[inline]
pub fn next_mode(byte: u8, mode: &State) -> Result<State, Error> {
    Ok(match (byte, mode) {
        // start string
        (b'"', _) => State::String,
        // ignored
        (b'\n' | b' ' | b'\r' | b'\t', _) => State::None,
        // object and array
        (b'{', _) => State::ObjectStart,
        (b':', State::None) => State::ColonSeparator,
        (b'}', _) => State::ObjectEnd,
        (b'[', _) => State::ArrayStart,
        (b']', _) => State::ArrayEnd,
        (b',', _) => State::ItemEnd,
        // null
        (b'n', _) => State::Null,
        // boolean(true)
        (b't', _) => State::Bool(true),
        // boolean(false)
        (b'f', _) => State::Bool(false),
        // number
        (b'0'..=b'9' | b'-', _) => State::Number,
        (b'.', _) => return Err(Error::OutOfSpec(OutOfSpecError::NumberWithTwoPeriods)),
        (token, _) => return Err(Error::OutOfSpec(OutOfSpecError::InvalidToken(token))),
    })
}

/// Typedef for the inside of an object.
pub type Object<'a> = BTreeMap<StringValue<'a>, Value<'a>>;

/// Reference to JSON data.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Value<'a> {
    /// A `null`
    Null,
    /// A string (i.e. something quoted; quotes are not part of this. Data has not been UTF-8 validated)
    String(StringValue<'a>),
    /// A number (i.e. something starting with a number with an optional period)
    Number(&'a [u8]),
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
/// If and only if there is not enough memory to allocate a vector of [`Value`].
pub fn parse(mut json: &[u8]) -> Result<Value, Error> {
    let mut mode = next_mode(json[0], &State::None)?;
    inner_parse(&mut json, &mut mode)
}

fn parse_string<'b, 'a>(
    values: &'b mut &'a [u8],
    mode: &mut State,
) -> Result<StringValue<'a>, Error> {
    let r = _parse_string(values)?;
    *mode = State::None;
    if values.is_empty() {
        return Err(Error::OutOfSpec(OutOfSpecError::InvalidEOF));
    }
    advance(values, mode)?;
    next_token(values, mode)?;
    Ok(r)
}

fn parse_number<'b, 'a>(values: &'b mut &'a [u8], mode: &mut State) -> Result<&'a [u8], Error> {
    let r = _parse_number(values);
    let byte = *values
        .get(0)
        .ok_or(Error::OutOfSpec(OutOfSpecError::InvalidEOF))?;
    *mode = next_mode(byte, mode)?;
    r
}

fn inner_parse<'b, 'a>(values: &'b mut &'a [u8], mode: &mut State) -> Result<Value<'a>, Error> {
    next_token(values, mode)?;
    match mode {
        State::ObjectStart => {
            *mode = State::None;
            parse_object(values, mode).map(Value::Object)
        }
        State::ArrayStart => parse_array(values, mode).map(Value::Array),
        State::String => parse_string(values, mode).map(Value::String),
        State::Number => parse_number(values, mode).map(Value::Number),
        State::Null => parse_null(values, mode).map(|_| Value::Null),
        State::Bool(true) => {
            parse_true(values, mode)?;
            Ok(Value::Bool(true))
        }
        State::Bool(false) => {
            parse_false(values, mode)?;
            Ok(Value::Bool(false))
        }
        _ => Err(Error::OutOfSpec(OutOfSpecError::InvalidToken(values[0]))),
    }
}

fn parse_object<'b, 'a>(values: &'b mut &'a [u8], mode: &mut State) -> Result<Object<'a>, Error> {
    let mut items = BTreeMap::new();
    loop {
        next_token(values, mode)?;
        if *mode == State::ItemEnd {
            *mode = State::None;
            advance(values, mode)?;
        }
        if *mode == State::ObjectEnd {
            *mode = State::None;
            advance(values, mode)?;
            break;
        }
        let (k, v) = parse_item(values, mode)?;
        items.insert(k, v);
    }
    Ok(items)
}

fn parse_array<'b, 'a>(
    values: &'b mut &'a [u8],
    mode: &mut State,
) -> Result<Vec<Value<'a>>, Error> {
    advance(values, mode)?;
    let mut items = vec![];
    loop {
        next_token(values, mode)?;
        if *mode == State::ArrayEnd {
            *mode = State::None;
            advance(values, mode)?;
            break;
        }
        items.push(inner_parse(values, mode)?);
        next_token(values, mode)?;
        if *mode == State::ItemEnd {
            *mode = State::None;
            advance(values, mode)?;
        }
    }
    Ok(items)
}

fn parse_item<'b, 'a>(
    values: &'b mut &'a [u8],
    mode: &mut State,
) -> Result<(StringValue<'a>, Value<'a>), Error> {
    next_token(values, mode)?;
    let key = parse_string(values, mode)?;

    next_token(values, mode)?;
    if *mode != State::ColonSeparator {
        return Err(Error::OutOfSpec(OutOfSpecError::KeyWithoutDoubleColon));
    }
    *mode = State::None;
    next_token(values, mode)?;
    let value = inner_parse(values, mode)?;
    next_token(values, mode)?;
    if !(*mode == State::ObjectEnd || *mode == State::ItemEnd) {
        return Err(Error::OutOfSpec(OutOfSpecError::KeyWithoutDoubleColon));
    }
    Ok((key, value))
}

#[inline]
pub fn advance(values: &mut &[u8], mode: &mut State) -> Result<(), Error> {
    *values = &values[1..];
    *mode = if let Some(byte) = values.get(0) {
        next_mode(*byte, mode)?
    } else {
        State::Finished
    };
    Ok(())
}

#[inline]
pub fn next_token(values: &mut &[u8], mode: &mut State) -> Result<(), Error> {
    loop {
        if values.is_empty() {
            break;
        }
        if *mode != State::None {
            break;
        }
        advance(values, mode)?;
    }
    Ok(())
}

#[inline]
fn parse_null(values: &mut &[u8], mode: &mut State) -> Result<(), Error> {
    _parse_null(values)?;
    let byte = *values
        .get(0)
        .ok_or(Error::OutOfSpec(OutOfSpecError::InvalidEOF))?;
    *mode = next_mode(byte, mode)?;
    Ok(())
}

#[inline]
fn parse_true(values: &mut &[u8], mode: &mut State) -> Result<(), Error> {
    _parse_true(values)?;
    let byte = *values
        .get(0)
        .ok_or(Error::OutOfSpec(OutOfSpecError::InvalidEOF))?;
    *mode = next_mode(byte, mode)?;
    Ok(())
}

#[inline]
fn parse_false(values: &mut &[u8], mode: &mut State) -> Result<(), Error> {
    _parse_false(values)?;
    let byte = *values
        .get(0)
        .ok_or(Error::OutOfSpec(OutOfSpecError::InvalidEOF))?;
    *mode = next_mode(byte, mode)?;
    Ok(())
}
