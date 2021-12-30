use super::error::*;
use super::lexer::{next_mode, State};

use alloc::string::ToString;
use alloc::vec::Vec;

/// Reference to JSON data.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Value<'a> {
    /// A `null`
    Null,
    /// A string (i.e. something quoted; quotes are not part of this. Data has not been UTF-8 validated)
    String(&'a [u8]),
    /// A Number (i.e. something starting with a number with an optional period)
    Number(&'a [u8]),
    /// A boolean (i.e. `false` or `true`)
    Boolean(bool),
    /// An object (i.e. items inside curly brackets `{}` separated by colon `:` and comma `,`)
    Object(Vec<(&'a [u8], Value<'a>)>),
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

fn inner_parse<'b, 'a>(values: &'b mut &'a [u8], mode: &mut State) -> Result<Value<'a>, Error> {
    next_token(values, mode)?;
    match mode {
        State::ObjectStart => {
            *mode = State::None;
            parse_object(values, mode).map(Value::Object)
        }
        State::ArrayStart => parse_array(values, mode).map(Value::Array),
        State::String | State::Escape => parse_string(values, mode).map(Value::String),
        State::Number(false) => parse_number(values, mode).map(Value::Number),
        State::Null(0) => parse_null(values, mode).map(|_| Value::Null),
        State::Boolean(true, 0) => {
            parse_true(values, mode)?;
            Ok(Value::Boolean(true))
        }
        State::Boolean(false, 0) => {
            parse_false(values, mode)?;
            Ok(Value::Boolean(false))
        }
        _ => Err(Error::OutOfSpec(format!("Unexpected token {}", values[0]))),
    }
}

fn parse_object<'b, 'a>(
    values: &'b mut &'a [u8],
    mode: &mut State,
) -> Result<Vec<(&'a [u8], Value<'a>)>, Error> {
    let mut items = vec![];
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
        items.push(parse_item(values, mode)?);
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
) -> Result<(&'a [u8], Value<'a>), Error> {
    next_token(values, mode)?;
    let key = parse_string(values, mode)?;

    next_token(values, mode)?;
    if *mode != State::ColonSeparator {
        return Err(Error::OutOfSpec(
            "The key of an object must terminate with :".to_string(),
        ));
    }
    *mode = State::None;
    next_token(values, mode)?;
    let value = inner_parse(values, mode)?;
    next_token(values, mode)?;
    if !(*mode == State::ObjectEnd || *mode == State::ItemEnd) {
        return Err(Error::OutOfSpec(
            "An item in an object must terminate with } or ,".to_string(),
        ));
    }
    Ok((key, value))
}

#[inline]
pub fn advance(values: &mut &[u8], mode: &mut State) -> Result<(), Error> {
    *values = &values[1..];
    if values.is_empty() {
        *mode = State::Finished
    } else {
        *mode = next_mode(values[0], mode)?;
    }
    Ok(())
}

#[inline]
pub fn next_token(values: &mut &[u8], mode: &mut State) -> Result<(), Error> {
    loop {
        if values.is_empty() {
            break;
        };
        if *mode != State::None {
            break;
        }
        advance(values, mode)?;
    }
    Ok(())
}

#[inline]
fn parse_string<'b, 'a>(values: &'b mut &'a [u8], mode: &mut State) -> Result<&'a [u8], Error> {
    debug_assert!(mode.is_string());
    let string = *values;
    let mut size = 0;
    loop {
        size += 1;
        advance(values, mode)?;
        if !mode.is_string() {
            break;
        }
    }
    advance(values, mode)?;
    next_token(values, mode)?;
    Ok(&string[1..size])
}

#[inline]
fn parse_number<'b, 'a>(values: &'b mut &'a [u8], mode: &mut State) -> Result<&'a [u8], Error> {
    debug_assert_eq!(*mode, State::Number(false));
    let string = *values;
    let mut size = 0;
    loop {
        size += 1;
        advance(values, mode)?;
        if !mode.is_number() {
            break;
        }
    }
    Ok(&string[..size])
}

#[inline]
fn parse_null(values: &mut &[u8], mode: &mut State) -> Result<(), Error> {
    debug_assert_eq!(*mode, State::Null(0));

    for _ in 0..3 {
        advance(values, mode)?;
    }
    assert_eq!(*mode, State::Null(3));
    next_token(values, mode)?;
    advance(values, mode)?;
    Ok(())
}

#[inline]
fn parse_true(values: &mut &[u8], mode: &mut State) -> Result<(), Error> {
    debug_assert_eq!(*mode, State::Boolean(true, 0));

    for _ in 0..3 {
        advance(values, mode)?;
    }
    assert_eq!(*mode, State::Boolean(true, 3));
    next_token(values, mode)?;
    advance(values, mode)?;
    Ok(())
}

#[inline]
fn parse_false(values: &mut &[u8], mode: &mut State) -> Result<(), Error> {
    debug_assert_eq!(*mode, State::Boolean(false, 0));

    for _ in 0..4 {
        advance(values, mode)?;
    }
    assert_eq!(*mode, State::Boolean(false, 4));
    next_token(values, mode)?;
    advance(values, mode)?;
    Ok(())
}
