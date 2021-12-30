use super::Error;

/// The state of the lexer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum State {
    Finished,
    None,              // something like a white space, that does not contribute
    Null(u8),          // the `null` (n,u,l,l)
    ObjectStart,       // {
    ObjectEnd,         // }
    ColonSeparator,    // :
    ItemEnd,           // ,
    ArrayStart,        // [
    ArrayEnd,          // ]
    Escape,            // \
    Boolean(bool, u8), // f,a,l,s,e or t,r,u,e
    String,            // something between double quotes
    Number(bool),      // whether it has a period or not
}

impl State {
    #[inline]
    pub fn is_string(&self) -> bool {
        self == &Self::Escape || self == &Self::String
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
        // string
        (b'"', State::String) => State::None,
        (b'"', _) => State::String,
        (92, State::String) => State::Escape,
        (_, State::Escape) => State::String,
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
        (b't', _) => State::Boolean(true, 0),
        (b'r', State::Boolean(true, 0)) => State::Boolean(true, 1),
        (b'u', State::Boolean(true, 1)) => State::Boolean(true, 2),
        (b'e', State::Boolean(true, 2)) => State::Boolean(true, 3),
        // boolean(false)
        (b'f', _) => State::Boolean(false, 0),
        (b'a', State::Boolean(false, 0)) => State::Boolean(false, 1),
        (b'l', State::Boolean(false, 1)) => State::Boolean(false, 2),
        (b's', State::Boolean(false, 2)) => State::Boolean(false, 3),
        (b'e', State::Boolean(false, 3)) => State::Boolean(false, 4),
        // number
        (b'0'..=b'9' | b'-' | b'E' | b'e', State::Number(is_float)) => State::Number(*is_float),
        (b'0'..=b'9' | b'-', _) => State::Number(false),
        (b'.', State::Number(false)) => State::Number(true),
        (b'.', _) => return Err(Error::OutOfSpec("Number with two periods".to_string())),
        (token, state) => {
            return Err(Error::OutOfSpec(format!(
                "Unexpected token {} for state {:?}",
                std::str::from_utf8(&[token]).unwrap(),
                state
            )))
        }
    })
}
