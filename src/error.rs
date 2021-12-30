/// Errors of this crate
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Error {
    /// When the data is not JSON compliant, together with the reason
    OutOfSpec(String),
}
