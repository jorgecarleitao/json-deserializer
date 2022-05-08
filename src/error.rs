use core::fmt::Display;

use alloc::string::String;

/// Errors of this crate
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Error {
    /// When the data is not JSON compliant, together with the reason
    OutOfSpec(String),
    /// Two utf-16 is still not implemented
    TwoUTF16SurrogatesNotYetImplemented,
}

impl Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}
