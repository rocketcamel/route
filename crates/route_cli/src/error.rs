use thiserror::Error;
use thiserror_ext::{Box, Construct};

use crate::config::errors::ConfigError;

#[derive(Error, Debug, Construct, Box)]
#[thiserror_ext(newtype(name = Error))]
pub enum ErrorKind {
    #[error("io error")]
    Io(#[from] std::io::Error),
    #[error("config")]
    Config(#[from] ConfigError),

    #[error(transparent)]
    Language(#[from] language::error::Error),
}

pub type Result<T> = core::result::Result<T, Error>;
