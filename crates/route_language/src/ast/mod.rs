pub mod ast;
mod parse;
mod visit;

use std::fmt::Display;

pub use parse::Parser;

use crate::ast::ast::Span;

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}
