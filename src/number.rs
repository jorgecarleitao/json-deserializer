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
        if matches!(state, State::FractionStart | State::ExponentSignedNegative) {
            is_float = true
        }

        length += 1;

        prev_state = state;

        if matches!(
            state,
            State::Signed | State::Zero | State::Nonzero | State::FractionStart | State::Fraction
        ) {
            number_end += 1;
        }

        *values = values.get(1..).ok_or(Error::InvalidEOF)?;

        if values.is_empty() {
            break;
        }

        let byte = values.get(0).ok_or(Error::InvalidEOF)?;

        state = next_state(*byte, state)?;

        if state == State::Finished {
            break;
        }
    }
    match prev_state {
        State::FractionStart => Err(Error::NumberWithEmptyFraction),
        State::ExponentStart | State::ExponentSignedPositive | State::ExponentSignedNegative => {
            Err(Error::NumberWithEmptyExponent)
        }
        _ => {
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
    }
}

/// The state of the string lexer
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum State {
    Finished, // when it is done
    Start,
    Signed,
    Zero,
    Nonzero,
    FractionStart,
    Fraction,
    ExponentStart,
    ExponentSignedPositive,
    ExponentSignedNegative,
    Exponent,
}

/// The transition state of the lexer
#[inline]
fn next_state(byte: u8, state: State) -> Result<State, Error> {
    Ok(match (byte, &state) {
        (b'-', State::Start) => State::Signed,
        (b'0', State::Start | State::Signed) => State::Zero,
        (b'1'..=b'9', State::Start | State::Signed) => State::Nonzero,

        (b'0'..=b'9', State::Zero) => return Err(Error::NumberWithLeadingZero),

        (b'.', State::Zero | State::Nonzero) => State::FractionStart,
        (b'e' | b'E', State::FractionStart) => return Err(Error::NumberWithEmptyFraction),
        (b'e' | b'E', State::Zero | State::Nonzero | State::Fraction) => State::ExponentStart,

        (b'0'..=b'9', State::Nonzero) => State::Nonzero,

        (b'0'..=b'9', State::FractionStart | State::Fraction) => State::Fraction,

        (b'+', State::ExponentStart) => State::ExponentSignedPositive,
        (b'-', State::ExponentStart) => State::ExponentSignedNegative,
        (
            b'0'..=b'9',
            State::ExponentStart
            | State::ExponentSignedPositive
            | State::ExponentSignedNegative
            | State::Exponent,
        ) => State::Exponent,

        (_, _) => State::Finished,
    })
}
