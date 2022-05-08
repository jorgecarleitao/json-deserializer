use super::error::*;

#[inline]
pub fn parse_null(values: &mut &[u8]) -> Result<(), Error> {
    let mut state = State::Null(0);
    loop {
        state = advance(values, state)?;
        if state == State::Finished {
            break;
        }
    }
    Ok(())
}

/// The state of the string lexer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum State {
    Finished, // when it is done
    Null(u8), // the `null` (n,u,l,l)
}

/// The transition state of the lexer
#[inline]
fn next_state(byte: u8, mode: State) -> Result<State, Error> {
    Ok(match (byte, mode) {
        (b'n', _) => State::Null(0),
        (b'u', State::Null(0)) => State::Null(1),
        (b'l', State::Null(1)) => State::Null(2),
        (b'l', State::Null(2)) => State::Finished,
        (token, _) => return Err(Error::OutOfSpec(OutOfSpecError::InvalidNullToken(token))),
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
