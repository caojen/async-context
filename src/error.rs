use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Error {
    ContextCancelled,
    ContextTimeout,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ContextCancelled => f.write_str("context cancelled"),
            Self::ContextTimeout => f.write_str("context timeout"),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = core::result::Result<T, Error>;
