use thiserror::Error;
use thiserror_ext::{Box, Construct};

#[derive(Error, Debug, Construct, Box)]
#[thiserror_ext(newtype(name = Error))]
pub enum ErrorKind {
    #[error("io error")]
    Io(#[from] std::io::Error),
    #[error("could not parse: {value}: line {line}:{col}")]
    Parse {
        value: String,
        line: usize,
        col: usize,
    },
}

pub type Result<T> = core::result::Result<T, Error>;
