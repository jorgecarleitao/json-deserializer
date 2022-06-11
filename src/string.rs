use alloc::borrow::Cow;

use alloc::string::String;

use super::Error;

#[inline]
fn skip_escape(values: &mut &[u8]) -> Result<usize, Error> {
    *values = &values[1..];
    let ch = *values.get(0).ok_or(Error::InvalidEOF)?;
    if ch == b'u' {
        const NUM_UNICODE_CHARS: usize = 4;
        if values.len() < NUM_UNICODE_CHARS {
            return Err(Error::InvalidEOF);
        } else {
            *values = &values[NUM_UNICODE_CHARS..];
        }
        Ok(NUM_UNICODE_CHARS + 1)
    } else {
        Ok(1)
    }
}

#[inline]
fn compute_length(values: &mut &[u8]) -> Result<(usize, usize), Error> {
    let mut length = 0;
    let mut escapes = 0;
    loop {
        *values = &values[1..];
        let ch = *values.get(0).ok_or(Error::InvalidEOF)?;
        length += 1;
        match ch {
            b'\\' => {
                escapes += 1;
                length += skip_escape(values)?;
            }
            b'"' => {
                *values = &values[1..];
                return Ok((length, escapes));
            }
            _ => {}
        }
    }
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
            .map_err(|_| Error::InvalidUtf8)
    }
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

#[inline]
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
                    return Err(Error::InvalidLoneLeadingSurrogateInHexEscape(n))
                }

                // Non-BMP characters are encoded as a sequence of two hex
                // escapes, representing UTF-16 surrogates. If deserializing a
                // utf-8 string the surrogates are required to be paired,
                // whereas deserializing a byte string accepts lone surrogates.
                n1 @ 0xD800..=0xDBFF => {
                    let byte = input.get(0).ok_or(Error::InvalidEOF)?;
                    if *byte == b'\\' {
                        input = &input[1..];
                    } else {
                        return Err(Error::UnexpectedEndOfHexEscape);
                    }

                    let byte = input.get(0).ok_or(Error::InvalidEOF)?;
                    if *byte == b'u' {
                        input = &input[1..];
                    } else {
                        return parse_escape(input, scratch);
                    }

                    let n2 = decode_hex_escape(input)?;
                    input = &input[4..];
                    if !(0xDC00..=0xDFFF).contains(&n2) {
                        return Err(Error::InvalidSurrogateInHexEscape(n2));
                    }

                    let n = (((n1 - 0xD800) as u32) << 10 | (n2 - 0xDC00) as u32) + 0x1_0000;
                    char::from_u32(n as u32).unwrap()
                }

                // Every u16 outside of the surrogate ranges above is guaranteed
                // to be a legal char.
                n => char::from_u32(n as u32).unwrap(),
            };

            scratch.push(c);
        }
        other => return Err(Error::InvalidEscaped(other)),
    }

    Ok(input)
}

fn decode_hex_escape(input: &[u8]) -> Result<u16, Error> {
    let numbers_u8: [u8; 4] = input[..4].try_into().unwrap();
    let mut n = 0;
    for number in numbers_u8 {
        let hex = decode_hex_val(number).ok_or(Error::InvalidHex(number))?;
        n = (n << 4) + hex;
    }
    Ok(n)
}
