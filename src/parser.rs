use super::error::*;
use super::lexer::{next_mode, State};

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

/// Typedef for the inside of an object.
pub type Object<'a> = BTreeMap<StringValue<'a>, Value<'a>>;

/// A JSON string, which may either be escaped and allocated
/// or non-escaped and a reference
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum StringValue<'a> {
    /// A JSON string with escaped characters replaced
    String(alloc::string::String),
    /// A JSON string without escaped characters
    Plain(&'a str),
}

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
        State::Bool(true, 0) => {
            parse_true(values, mode)?;
            Ok(Value::Bool(true))
        }
        State::Bool(false, 0) => {
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

fn compute_length(values: &mut &[u8], mode: &mut State) -> Result<(usize, usize), Error> {
    let mut length = 0;
    let mut escapes = 0;
    loop {
        length += 1;
        advance(values, mode)?;
        if *mode == State::Escape {
            escapes += 1
        };
        if !mode.is_string() {
            break;
        }
    }
    advance(values, mode)?;
    next_token(values, mode)?;

    Ok((length, escapes))
}

#[inline]
fn parse_string<'b, 'a>(
    values: &'b mut &'a [u8],
    mode: &mut State,
) -> Result<StringValue<'a>, Error> {
    // compute the size of the string value and whether it has escapes
    debug_assert!(mode.is_string());
    let string = *values;
    let (length, escapes) = compute_length(values, mode)?;

    let mut data = &string[1..length];
    if escapes > 0 {
        let capacity = data.len() - escapes;
        let mut container = String::with_capacity(capacity);

        while !data.is_empty() {
            let first = data[0];
            if first == b'\\' {
                data = &data[1..];
                data = parse_escape(data, &mut container)?;
            } else {
                container.push(first as char);
                data = &data[1..];
            }
        }
        Ok(StringValue::String(container))
    } else {
        alloc::str::from_utf8(data)
            .map(StringValue::Plain)
            .map_err(|_| Error::OutOfSpec(OutOfSpecError::InvalidUtf8))
    }
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
    debug_assert_eq!(*mode, State::Bool(true, 0));

    for _ in 0..3 {
        advance(values, mode)?;
    }
    assert_eq!(*mode, State::Bool(true, 3));
    next_token(values, mode)?;
    advance(values, mode)?;
    Ok(())
}

#[inline]
fn parse_false(values: &mut &[u8], mode: &mut State) -> Result<(), Error> {
    debug_assert_eq!(*mode, State::Bool(false, 0));

    for _ in 0..4 {
        advance(values, mode)?;
    }
    assert_eq!(*mode, State::Bool(false, 4));
    next_token(values, mode)?;
    advance(values, mode)?;
    Ok(())
}

fn encode_surrogate(n: u16) -> [u8; 3] {
    [
        (n >> 12 & 0b0000_1111) as u8 | 0b1110_0000,
        (n >> 6 & 0b0011_1111) as u8 | 0b1000_0000,
        (n & 0b0011_1111) as u8 | 0b1000_0000,
    ]
}

#[allow(clippy::zero_prefixed_literal)]
static HEX: [u8; 256] = {
    const __: u8 = 255; // not a hex digit
    [
        //   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 0
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 1
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 2
        00, 01, 02, 03, 04, 05, 06, 07, 08, 09, __, __, __, __, __, __, // 3
        __, 10, 11, 12, 13, 14, 15, __, __, __, __, __, __, __, __, __, // 4
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 5
        __, 10, 11, 12, 13, 14, 15, __, __, __, __, __, __, __, __, __, // 6
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 7
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 8
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 9
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // A
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // B
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // C
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // D
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // E
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // F
    ]
};

fn decode_hex_val(val: u8) -> Option<u16> {
    let n = HEX[val as usize] as u16;
    if n == 255 {
        None
    } else {
        Some(n)
    }
}

/// Parses a JSON escape sequence and appends it into the scratch space. Assumes
/// the previous byte read was a backslash.
fn parse_escape<'a>(mut input: &'a [u8], scratch: &mut String) -> Result<&'a [u8], Error> {
    let ch = input[0];
    input = &input[1..];
    match ch {
        b'"' => scratch.push('"'),
        b'\\' => scratch.push('\\'),
        b'/' => scratch.push('/'),
        b'b' => scratch.push('\x08'),
        b'f' => scratch.push('\x0c'),
        b'n' => scratch.push('\n'),
        b'r' => scratch.push('\r'),
        b't' => scratch.push('\t'),
        b'u' => {
            let hex = decode_hex_escape(input)?;
            input = &input[4..];

            let c = match hex {
                n @ 0xDC00..=0xDFFF => {
                    let bytes = encode_surrogate(n);

                    scratch.push_str(alloc::str::from_utf8(&bytes).unwrap());

                    return Ok(input);
                }

                // Non-BMP characters are encoded as a sequence of two hex
                // escapes, representing UTF-16 surrogates. If deserializing a
                // utf-8 string the surrogates are required to be paired,
                // whereas deserializing a byte string accepts lone surrogates.
                _n1 @ 0xD800..=0xDBFF => {
                    return Err(Error::TwoUTF16SurrogatesNotYetImplemented);
                    /*
                    if tri!(peek_or_eof(read)) == b'\\' {
                        read.discard();
                    } else {
                        return if validate {
                            error(read, ErrorCode::UnexpectedEndOfHexEscape)
                        } else {
                            encode_surrogate(scratch, n1);
                            Ok(())
                        };
                    }

                    if tri!(peek_or_eof(read)) == b'u' {
                        read.discard();
                    } else {
                        return if validate {
                            error(read, ErrorCode::UnexpectedEndOfHexEscape)
                        } else {
                            encode_surrogate(scratch, n1);
                            // The \ prior to this byte started an escape sequence,
                            // so we need to parse that now. This recursive call
                            // does not blow the stack on malicious input because
                            // the escape is not \u, so it will be handled by one
                            // of the easy nonrecursive cases.
                            parse_escape(read, validate, scratch)
                        };
                    }

                    let n2 = tri!(read.decode_hex_escape());

                    if n2 < 0xDC00 || n2 > 0xDFFF {
                        return error(read, ErrorCode::LoneLeadingSurrogateInHexEscape);
                    }

                    let n = (((n1 - 0xD800) as u32) << 10 | (n2 - 0xDC00) as u32) + 0x1_0000;

                    match char::from_u32(n) {
                        Some(c) => c,
                        None => {
                            return error(read, ErrorCode::InvalidUnicodeCodePoint);
                        }
                    }
                    */
                }

                // Every u16 outside of the surrogate ranges above is guaranteed
                // to be a legal char.
                n => char::from_u32(n as u32).unwrap(),
            };

            scratch.push(c);
        }
        other => return Err(Error::OutOfSpec(OutOfSpecError::InvalidEscaped(other))),
    }

    Ok(input)
}

fn decode_hex_escape(input: &[u8]) -> Result<u16, Error> {
    let numbers_u8: [u8; 4] = input[..4].try_into().unwrap();
    let mut n = 0;
    for number in numbers_u8 {
        let hex =
            decode_hex_val(number).ok_or(Error::OutOfSpec(OutOfSpecError::InvalidHex(number)))?;
        n = (n << 4) + hex;
    }
    Ok(n)
}
