use thiserror::Error;
use thiserror_ext::{Box, Construct};

use crate::ast::ast::Span;

#[derive(Error, Construct, Box, Debug)]
#[thiserror_ext(newtype(name = Error))]
pub enum ErrorKind {
    #[error("parse error: {why} at {span}")]
    Parse { why: String, span: Span },
}

pub type Result<T> = core::result::Result<T, Error>;
