#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Error {
    OutOfSpec(String),
}
