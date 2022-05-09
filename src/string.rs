use std::borrow::Cow;

use alloc::string::String;

use super::{Error, OutOfSpecError};

/// The state of the string lexer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum State {
    Finished,      // when it is done
    Escape,        // \
    String,        // something between double quotes
    Codepoint(u8), // parsing \uXXXX (0 => u, 1-3 => X)
}

/// The transition state of the lexer
#[inline]
fn next_mode(byte: u8, mode: State) -> Result<State, Error> {
    Ok(match (byte, mode) {
        (_, State::Codepoint(0)) => State::Codepoint(1),
        (_, State::Codepoint(1)) => State::Codepoint(2),
        (_, State::Codepoint(2)) => State::Codepoint(3),
        (_, State::Codepoint(3)) => State::String,
        (b'"', State::String) => State::Finished,
        (b'u', State::Escape) => State::Codepoint(0),
        (_, State::Escape) => State::String,
        (b'\\', State::String) => State::Escape,
        (_, State::String) => mode,
        (token, _) => return Err(Error::OutOfSpec(OutOfSpecError::InvalidStringToken(token))),
    })
}

#[inline]
fn advance(values: &mut &[u8], mode: State) -> Result<State, Error> {
    *values = &values[1..];
    let byte = values
        .get(0)
        .ok_or(Error::OutOfSpec(OutOfSpecError::InvalidEOF))?;
    next_mode(*byte, mode)
}

fn compute_length(values: &mut &[u8]) -> Result<(usize, usize), Error> {
    let mut length = 0;
    let mut escapes = 0;
    let mut mode = State::String;
    loop {
        length += 1;
        mode = advance(values, mode)?;
        if mode == State::Escape {
            escapes += 1
        };
        if mode == State::Finished {
            *values = &values[1..];
            break;
        }
    }
    Ok((length, escapes))
}

#[inline]
pub fn parse_string<'b, 'a>(values: &'b mut &'a [u8]) -> Result<Cow<'a, str>, Error> {
    // compute the size of the string value and whether it has escapes
    let string = *values;
    let (length, escapes) = compute_length(values)?;

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
        Ok(Cow::Owned(container))
    } else {
        alloc::str::from_utf8(data)
            .map(Cow::Borrowed)
            .map_err(|_| Error::OutOfSpec(OutOfSpecError::InvalidUtf8))
    }
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
