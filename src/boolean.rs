use super::error::*;

/// The state of the string lexer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum State {
    Finished, // when it is done
    Bool(u8), // the `null` (n,u,l,l)
}

/// The transition state of the lexer
#[inline]
fn next_state_true(byte: u8, mode: State) -> Result<State, Error> {
    Ok(match (byte, mode) {
        (b't', _) => State::Bool(0),
        (b'r', State::Bool(0)) => State::Bool(1),
        (b'u', State::Bool(1)) => State::Bool(2),
        (b'e', State::Bool(2)) => State::Finished,
        (token, _) => return Err(Error::OutOfSpec(OutOfSpecError::InvalidTrueToken(token))),
    })
}

#[inline]
fn advance_true(values: &mut &[u8], state: State) -> Result<State, Error> {
    *values = &values[1..];
    let byte = values
        .get(0)
        .ok_or(Error::OutOfSpec(OutOfSpecError::InvalidEOF))?;
    next_state_true(*byte, state)
}

#[inline]
pub fn parse_true(values: &mut &[u8]) -> Result<(), Error> {
    let mut state = State::Bool(0);
    loop {
        state = advance_true(values, state)?;
        if state == State::Finished {
            break;
        }
    }
    Ok(())
}

/// The transition state of the lexer
#[inline]
fn next_state_false(byte: u8, mode: State) -> Result<State, Error> {
    Ok(match (byte, mode) {
        (b'f', _) => State::Bool(0),
        (b'a', State::Bool(0)) => State::Bool(1),
        (b'l', State::Bool(1)) => State::Bool(2),
        (b's', State::Bool(2)) => State::Bool(3),
        (b'e', State::Bool(3)) => State::Finished,
        (token, _) => return Err(Error::OutOfSpec(OutOfSpecError::InvalidFalseToken(token))),
    })
}

#[inline]
fn advance_false(values: &mut &[u8], state: State) -> Result<State, Error> {
    *values = &values[1..];
    let byte = values
        .get(0)
        .ok_or(Error::OutOfSpec(OutOfSpecError::InvalidEOF))?;
    next_state_false(*byte, state)
}

#[inline]
pub fn parse_false(values: &mut &[u8]) -> Result<(), Error> {
    let mut state = State::Bool(0);
    loop {
        state = advance_false(values, state)?;
        if state == State::Finished {
            break;
        }
    }
    Ok(())
}
