mod ast;
mod parse;

use crate::{
    ast::ast::{Span, Token, TokenKind},
    error::{Error, Result},
};

pub use parse::Parser;

pub struct Lexer<'a> {
    source: &'a [u8],
    len: usize,
    pos: usize,
    line: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a [u8]) -> Self {
        Self {
            source,
            len: source.len(),
            pos: 0,
            line: 0,
        }
    }

    fn peek(&self) -> u8 {
        if self.pos == self.len {
            return 0;
        }
        return self.source[self.pos];
    }

    fn peek_2(&self) -> u8 {
        if self.pos + 1 >= self.len {
            return 0;
        }
        return self.source[self.pos + 1];
    }

    fn bump(&mut self) {
        self.pos = self.len.min(self.pos + 1)
    }

    fn bump_peek(&mut self) -> u8 {
        self.bump();
        return self.peek();
    }

    fn read_kind(&mut self) -> TokenKind {
        let mut c = self.peek();
        if c == 0 {
            return TokenKind::Eof;
        }

        match c {
            b'-' => {
                self.bump();
                c = self.peek();
                if c == b'>' {
                    self.bump();
                    return TokenKind::Arrow;
                } else {
                    return TokenKind::Error;
                }
            }
            b'{' => {
                self.bump();
                TokenKind::LBrace
            }
            b'}' => {
                self.bump();
                TokenKind::RBrace
            }
            b':' => {
                self.bump();
                TokenKind::Colon
            }
            b'=' => {
                self.bump();
                TokenKind::Equals
            }
            mut c if c.is_ascii_digit() => {
                loop {
                    c = self.bump_peek();

                    if !(c.is_ascii_digit() || c == b'.' || c == b'_') {
                        break;
                    }
                }
                TokenKind::Number
            }
            mut c if is_alpha(c) => {
                let start = self.pos;
                loop {
                    c = self.bump_peek();

                    if !(is_alpha(c) || c.is_ascii_digit()) {
                        break;
                    }
                }
                let value = str::from_utf8(&self.source[start..self.pos]).unwrap();
                match value {
                    "true" => TokenKind::True,
                    "false" => TokenKind::False,
                    "nil" => TokenKind::Nil,
                    "tcp" => TokenKind::Tcp,
                    "let" => TokenKind::Let,
                    _ => TokenKind::Identifier,
                }
            }
            c if is_whitespace(c) => {
                println!("whitespace");
                self.bump();
                if c == b'\n' {
                    self.line += 1
                }
                TokenKind::Whitespace
            }
            _ => TokenKind::Error,
        }
    }

    pub fn next_token(&mut self) -> Result<Token<'a>> {
        let mut start = self.pos;
        let mut initial_line = self.line;
        let mut kind = self.read_kind();

        while kind == TokenKind::Whitespace {
            start = self.pos;
            initial_line = self.line;
            kind = self.read_kind();
        }

        if kind == TokenKind::Error {
            let value = str::from_utf8(&self.source[start..=self.pos]).unwrap();
            return Err(Error::parse(
                format!("{value} into a token"),
                initial_line + 1,
                self.pos - start,
            ));
        }
        let text = str::from_utf8(&self.source[start..self.pos]).unwrap();
        println!("token: {text}");

        Ok(Token {
            kind,
            text,
            span: Span {
                start: start,
                end: self.pos,
                line: self.line,
            },
        })
    }
}

fn is_alpha(c: u8) -> bool {
    return c.is_ascii_alphabetic() || c == b'.' || c == b'-' || c == b'_';
}

fn is_whitespace(c: u8) -> bool {
    return c.is_ascii_whitespace();
}
