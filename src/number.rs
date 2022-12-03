use super::error::*;
use super::Number;

#[inline]
pub fn parse_number<'b, 'a>(values: &'b mut &'a [u8]) -> Result<Number<'a>, Error> {
    let number = *values;

    let mut is_float = false;
    let mut number_end = 0;
    let mut length = 0;

    let mut prev_state = State::Start;
    let byte = values.get(0).ok_or(Error::InvalidEOF)?;
    let mut state = next_state(*byte, prev_state)?;

    loop {
        if let State::Fraction = state {
            is_float = true
        }
        length += 1;

        prev_state = state;

        if matches!(prev_state, State::Number | State::Fraction) {
            number_end += 1
        };

        if values.len() == 1 {
            break;
        }

        *values = values.get(1..).ok_or(Error::InvalidEOF)?;
        let byte = values.get(0).ok_or(Error::InvalidEOF)?;

        state = next_state(*byte, state)?;

        if state == State::Finished {
            break;
        }
    }
    let number = &number[..length];
    let exponent = if number_end == number.len() {
        &[]
    } else {
        &number[number_end + 1..]
    };
    let number = &number[..number_end];
    Ok(if is_float {
        Number::Float(number, exponent)
    } else {
        Number::Integer(number, exponent)
    })
}

/// The state of the string lexer
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum State {
    Finished, // when it is done
    Start,
    Number,
    Fraction,
    Exponent,
}

/// The transition state of the lexer
#[inline]
fn next_state(byte: u8, state: State) -> Result<State, Error> {
    Ok(match (byte, &state) {
        (b'0'..=b'9' | b'-', State::Start) => State::Number,
        (b'.', State::Number) => State::Fraction,
        (b'0'..=b'9', State::Number | State::Fraction) => state,
        (b'E' | b'e', State::Number | State::Fraction) => State::Exponent,
        (b'0'..=b'9' | b'-' | b'+', State::Exponent) => State::Exponent,
        (b'E' | b'e' | b'.' | b'-', _) => return Err(Error::NumberWithTwoPeriods),
        (_, _) => State::Finished,
    })
}
