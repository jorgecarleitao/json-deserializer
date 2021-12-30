/*
{"a": "b"}
["a", "b"]
"a"
null
""
*/
use super::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mode {
    Finished,
    None,           // something like a white space, that does not contribute
    Null(u8),       // the `null`
    ObjectStart,    // {
    ObjectEnd,      // }
    ColonSeparator, // :
    ItemEnd,        // ,
    ArrayStart,     // [
    ArrayEnd,       // ]
    Escape,
    Boolean(bool, u8),
    String,
    Number(bool), // whether it has a period or not
}

impl Mode {
    #[inline]
    pub fn is_string(&self) -> bool {
        self == &Self::Escape || self == &Self::String
    }

    #[inline]
    pub fn is_number(&self) -> bool {
        self == &Self::Number(true) || self == &Self::Number(false)
    }
}

#[inline]
pub fn next_mode(byte: u8, mode: &Mode) -> Result<Mode, Error> {
    println!("{}:{}", byte, std::str::from_utf8(&[byte]).unwrap());
    Ok(match (byte, mode) {
        (92, Mode::String) => Mode::Escape,
        (_, Mode::Escape) => Mode::String,
        (b'{', Mode::None) => Mode::ObjectStart,
        (b':', Mode::None) => Mode::ColonSeparator,
        (b'}', _) => Mode::ObjectEnd,
        (b'[', _) => Mode::ArrayStart,
        (b']', _) => Mode::ArrayEnd,
        (b'"', Mode::String) => Mode::None,
        (b'"', _) => Mode::String,
        (b'n', m) if *m != Mode::String => Mode::Null(0),
        (b'u', Mode::Null(0)) => Mode::Null(1),
        (b'l', Mode::Null(1)) => Mode::Null(2),
        (b'l', Mode::Null(2)) => Mode::Null(3),
        (b't', m) if *m != Mode::String => Mode::Boolean(true, 0),
        (b'r', Mode::Boolean(true, 0)) => Mode::Boolean(true, 1),
        (b'u', Mode::Boolean(true, 1)) => Mode::Boolean(true, 2),
        (b'e', Mode::Boolean(true, 2)) => Mode::Boolean(true, 3),
        (b'f', m) if *m != Mode::String => Mode::Boolean(false, 0),
        (b'a', Mode::Boolean(false, 0)) => Mode::Boolean(false, 1),
        (b'l', Mode::Boolean(false, 1)) => Mode::Boolean(false, 2),
        (b's', Mode::Boolean(false, 2)) => Mode::Boolean(false, 3),
        (b'e', Mode::Boolean(false, 3)) => Mode::Boolean(false, 4),
        (b'.', Mode::Number(false)) => Mode::Number(true),
        (b'.', _) => return Err(Error::OutOfSpec("The ".to_string())),
        (b'0'..=b'9', Mode::Number(is_float)) => Mode::Number(*is_float),
        (b'0'..=b'9', _) => Mode::Number(false),
        (b',', _) => Mode::ItemEnd,
        (_, Mode::Null(_)) => Mode::None,
        (_, Mode::Boolean(_, _)) => Mode::None,
        // todo:
        _ => *mode,
    })
}
