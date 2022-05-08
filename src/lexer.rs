use super::Error;

use alloc::string::ToString;

/// The state of the lexer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum State {
    Finished,
    None,           // something like a white space, that does not contribute
    Null(u8),       // the `null` (n,u,l,l)
    ObjectStart,    // {
    ObjectEnd,      // }
    ColonSeparator, // :
    ItemEnd,        // ,
    ArrayStart,     // [
    ArrayEnd,       // ]
    Escape,         // \
    Bool(bool, u8), // f,a,l,s,e or t,r,u,e
    String,         // something between double quotes
    Codepoint(u8),  // parsing \uXXXX (0 => u, 1-3 => X)
    Number(bool),   // whether it has a period or not
}

impl State {
    #[inline]
    pub fn is_string(&self) -> bool {
        self == &Self::Escape || self == &Self::String || matches!(self, Self::Codepoint(_))
    }

    #[inline]
    pub fn is_number(&self) -> bool {
        self == &Self::Number(true) || self == &Self::Number(false)
    }
}

/// The transition state of the lexer
#[inline]
pub fn next_mode(byte: u8, mode: &State) -> Result<State, Error> {
    Ok(match (byte, mode) {
        (_, State::Codepoint(0)) => State::Codepoint(1),
        (_, State::Codepoint(1)) => State::Codepoint(2),
        (_, State::Codepoint(2)) => State::Codepoint(3),
        (_, State::Codepoint(3)) => State::String,
        // finish string
        (b'"', State::String) => State::None,
        (b'u', State::Escape) => State::Codepoint(0),
        (_, State::Escape) => State::String,
        // start string
        (b'"', _) => State::String,
        // start escape
        (92, State::String) => State::Escape,
        (_, State::String) => *mode,
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
        (b'n', _) => State::Null(0),
        (b'u', State::Null(0)) => State::Null(1),
        (b'l', State::Null(1)) => State::Null(2),
        (b'l', State::Null(2)) => State::Null(3),
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
        (b'0'..=b'9' | b'-' | b'E' | b'e', State::Number(is_float)) => State::Number(*is_float),
        (b'0'..=b'9' | b'-', _) => State::Number(false),
        (b'.', State::Number(false)) => State::Number(true),
        (b'.', _) => return Err(Error::OutOfSpec("Number with two periods".to_string())),
        (token, state) => {
            return Err(Error::OutOfSpec(format!(
                "Unexpected token {} for state {:?}",
                alloc::str::from_utf8(&[token]).unwrap(),
                state
            )))
        }
    })
}
