use core::fmt::Display;

/// List of possible errors
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Error {
    /// todo
    NumberWithTwoPeriods,
    /// todo
    InvalidUtf8,
    /// todo
    InvalidEscaped(u8),
    /// todo
    InvalidHex(u8),
    /// Invalid surrogate
    InvalidLoneLeadingSurrogateInHexEscape(u16),
    /// Invalid surrogate pair
    InvalidSurrogateInHexEscape(u16),
    /// When a surrogate misses the pair
    UnexpectedEndOfHexEscape,
    /// todo
    KeyWithoutDoubleColon,
    /// todo
    InvalidToken(u8),
    /// todo
    MissingComma(u8),
    /// todo
    InvalidStringToken(u8),
    /// todo
    InvalidNullToken([u8; 4]),
    /// When an invalid token is found while trying to parse "false"
    InvalidFalseToken([u8; 5]),
    /// When an invalid token is found while trying to parse "true"
    InvalidTrueToken([u8; 4]),
    /// todo
    InvalidEOF,
}

impl Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}
