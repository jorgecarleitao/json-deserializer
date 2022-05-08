use super::error::*;

#[inline]
pub fn parse_number<'b, 'a>(values: &'b mut &'a [u8]) -> Result<&'a [u8], Error> {
    let string = *values;
    let mut length = 0;
    let mut state = State::Number(false);
    loop {
        length += 1;
        state = advance(values, state)?;
        if state == State::Finished {
            break;
        }
    }
    Ok(&string[..length])
}

/// The state of the string lexer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum State {
    Finished,     // when it is done
    Number(bool), // something between double quotes
}

/// The transition state of the lexer
#[inline]
fn next_state(byte: u8, mode: State) -> Result<State, Error> {
    Ok(match (byte, mode) {
        // number
        (b'0'..=b'9' | b'-' | b'E' | b'e', State::Number(_)) => mode,
        (b'0'..=b'9' | b'-', _) => State::Number(false),
        (b'.', State::Number(false)) => State::Number(true),
        (b'.', _) => return Err(Error::OutOfSpec(OutOfSpecError::NumberWithTwoPeriods)),
        (_, _) => State::Finished,
    })
}

#[inline]
fn advance(values: &mut &[u8], state: State) -> Result<State, Error> {
    *values = &values[1..];
    let byte = values
        .get(0)
        .ok_or(Error::OutOfSpec(OutOfSpecError::InvalidEOF))?;
    next_state(*byte, state)
}
