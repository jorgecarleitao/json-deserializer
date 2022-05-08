use super::{Error, OutOfSpecError};

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
    Bool(bool, u8), // f,a,l,s,e or t,r,u,e
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
        (b't', _) => State::Bool(true, 0),
        (b'r', State::Bool(true, 0)) => State::Bool(true, 1),
        (b'u', State::Bool(true, 1)) => State::Bool(true, 2),
        (b'e', State::Bool(true, 2)) => State::Bool(true, 3),
        // boolean(false)
        (b'f', _) => State::Bool(false, 0),
        (b'a', State::Bool(false, 0)) => State::Bool(false, 1),
        (b'l', State::Bool(false, 1)) => State::Bool(false, 2),
        (b's', State::Bool(false, 2)) => State::Bool(false, 3),
        (b'e', State::Bool(false, 3)) => State::Bool(false, 4),
        // number
        (b'0'..=b'9' | b'-', _) => State::Number,
        (b'.', _) => return Err(Error::OutOfSpec(OutOfSpecError::NumberWithTwoPeriods)),
        (token, _) => return Err(Error::OutOfSpec(OutOfSpecError::InvalidToken(token))),
    })
}
