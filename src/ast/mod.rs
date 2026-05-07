use crate::error::{Error, Result};

#[allow(unused)]
#[derive(PartialEq, Debug)]
pub enum Token {
    // symbols
    Arrow,
    LBrace,
    RBrace,
    Colon,
    Identifier(String),
    Keyword(Kw),
    Number(usize),
    // whitespace
    Whitespace,
    Comment,
    // line endings
    Eof,
    Newline,
    Error,
}

#[allow(unused)]
#[derive(PartialEq, Debug)]
pub enum Kw {
    Namespace,
    Gateway,
}

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

    fn read_while(&mut self, pred: fn(u8) -> bool) -> &'a [u8] {
        let start = self.pos;
        while pred(self.peek()) {
            self.bump();
        }
        &self.source[start..self.pos]
    }

    fn read_kind(&mut self) -> Token {
        let mut c = self.peek();
        if c == 0 {
            return Token::Eof;
        }

        match c {
            b'-' => {
                self.bump();
                c = self.peek();
                if c == b'>' {
                    self.bump();
                    return Token::Arrow;
                } else {
                    return Token::Error;
                }
            }
            b'{' => {
                self.bump();
                Token::LBrace
            }
            b'}' => {
                self.bump();
                Token::RBrace
            }
            b':' => {
                self.bump();
                Token::Colon
            }
            mut c if is_alpha(c) => {
                println!("alpha");
                let start = self.pos;
                loop {
                    c = self.bump_peek();

                    if !(is_alpha(c) || c.is_ascii_digit()) {
                        break;
                    }
                }
                let value = str::from_utf8(&self.source[start..self.pos]).unwrap();
                match value {
                    "namespace" => return Token::Keyword(Kw::Namespace),
                    "gateway" => return Token::Keyword(Kw::Gateway),
                    _ => {}
                }

                Token::Identifier(value.to_string())
            }
            c if is_whitespace(c) => {
                println!("whitespace");
                self.bump();
                if c == b'\n' {
                    self.line += 1
                }
                Token::Whitespace
            }
            _ => Token::Error,
        }
    }

    pub fn next_token(&mut self) -> Result<Token> {
        let mut start = self.pos;
        let mut initial_line = self.line;
        let mut kind = self.read_kind();

        while kind == Token::Whitespace {
            start = self.pos;
            initial_line = self.line;
            kind = self.read_kind();
        }

        if kind == Token::Error {
            let value = str::from_utf8(&self.source[start..=self.pos]).unwrap();
            return Err(Error::parse(value, initial_line + 1, self.pos - start));
        }

        Ok(kind)
    }
}

fn is_alpha(c: u8) -> bool {
    return c.is_ascii_alphanumeric() || c == b'.' || c == b'-' || c == b'_';
}

fn is_whitespace(c: u8) -> bool {
    return c.is_ascii_whitespace();
}
