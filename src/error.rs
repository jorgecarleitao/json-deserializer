use core::fmt::Display;

/// List of possible errors
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OutOfSpecError {
    /// todo
    NumberWithTwoPeriods,
    /// todo
    InvalidUtf8,
    /// todo
    InvalidEscaped(u8),
    /// todo
    InvalidHex(u8),
    /// todo
    KeyWithoutDoubleColon,
    /// todo
    InvalidToken(u8),
}

/// Errors of this crate
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Error {
    /// When the data is not JSON compliant, together with the reason
    OutOfSpec(OutOfSpecError),
    /// Two utf-16 is still not implemented
    TwoUTF16SurrogatesNotYetImplemented,
}

impl Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}
