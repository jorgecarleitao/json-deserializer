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
    Bool(bool),     // f,a,l,s,e or t,r,u,e
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
        (b't', _) => State::Bool(true),
        // boolean(false)
        (b'f', _) => State::Bool(false),
        // number
        (b'0'..=b'9' | b'-', _) => State::Number,
        (b'.', _) => return Err(Error::OutOfSpec(OutOfSpecError::NumberWithTwoPeriods)),
        (token, _) => return Err(Error::OutOfSpec(OutOfSpecError::InvalidToken(token))),
    })
}
