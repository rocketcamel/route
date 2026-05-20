use crate::{
    ast::ast::{Assign, Ast, Block, Expression, Span, Statement, Token, TokenKind},
    error::{Error, Result},
};

struct Lexer<'a> {
    input: &'a [u8],
    pos: usize,
    len: usize,
    line: usize,
}

struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: Token<'a>,
    current_kind: TokenKind,
    lookahead_token: Token<'a>,
    lookahead_kind: TokenKind,
}

fn is_alpha(char: u8) -> bool {
    return char.is_ascii_alphabetic() || char == b'.' || char == b'-' || char == b'_';
}

fn is_whitespace(char: u8) -> bool {
    return char.is_ascii_whitespace();
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a [u8]) -> Self {
        Self {
            input,
            pos: 0,
            len: input.len(),
            line: 1,
        }
    }

    fn peek(&mut self) -> u8 {
        if self.pos == self.len {
            return 0;
        }
        self.input[self.pos]
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
                    TokenKind::Arrow
                } else {
                    TokenKind::Error
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
            mut c if c.is_ascii_alphanumeric() => {
                loop {
                    c = self.bump_peek();

                    if !(c.is_ascii_alphanumeric() || c == b'.' || c == b'_') {
                        break;
                    }
                }
                TokenKind::Number
            }
            mut c if is_alpha(c) => {
                let start = self.pos;
                loop {
                    c = self.bump_peek();

                    if !is_alpha(c) {
                        break;
                    }
                }
                let value = str::from_utf8(&self.input[start..self.pos]).unwrap();
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
                self.bump();
                if c == b'\n' {
                    self.line += 1
                }
                TokenKind::Whitespace
            }
            _ => TokenKind::Error,
        }
    }

    fn next_token(&mut self) -> Result<Token<'a>> {
        let mut start = self.pos;
        let mut initial_line = self.line;
        let mut kind = self.read_kind();

        if kind == TokenKind::Whitespace {
            start = self.pos;
            initial_line = self.line;
            kind = self.read_kind();
        }

        if kind == TokenKind::Error {
            let value = str::from_utf8(&self.input[start..=self.pos]).unwrap();
            return Err(Error::parse(
                format!("{value} into a token"),
                initial_line,
                self.pos - start,
            ));
        }
        let text = str::from_utf8(&self.input[start..self.pos]).unwrap();

        Ok(Token {
            kind,
            text,
            span: Span {
                start,
                end: self.pos,
                line: initial_line,
            },
        })
    }
}

fn display<'a>(token: Token<'a>) -> String {
    let kind = token.kind;

    if kind == TokenKind::Identifier {
        token.text.to_string()
    } else if kind == TokenKind::Error {
        format!("error {}", token.text)
    } else {
        format!("{kind:?}")
    }
}

impl<'a> Expression<'a> {
    pub fn span(&self) -> Span {
        match self {
            Expression::Boolean(node)
            | Expression::Nil(node)
            | Expression::Number(node)
            | Expression::String(node => node.span,
        }
    }
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a [u8]) -> Result<Self> {
        let mut lexer = Lexer::new(input);
        let current_token = lexer.next_token()?;
        let current_kind = current_token.kind;
        let lookahead_token = lexer.next_token()?;
        let lookahead_kind = lookahead_token.kind;

        Ok(Self {
            current_token,
            current_kind,
            lookahead_token,
            lookahead_kind,
            lexer,
        })
    }

    pub fn parse(&mut self) -> Result<Ast<'a>> {
        Ok(Ast {
            block: self.parse_block_node()?,
        })
    }

    fn current_is(&self, kind: TokenKind) -> bool {
        return self.current_kind == kind;
    }

    fn lookahead_is(&self, kind: TokenKind) -> bool {
        return self.lookahead_kind == kind;
    }

    fn consume(&mut self) -> Result<Token<'a>> {
        let old_token = self.current_token;
        self.current_token = self.lookahead_token;
        self.current_kind = self.lookahead_kind;
        self.lookahead_token = self.lexer.next_token()?;
        self.lookahead_kind = self.lookahead_token.kind;
        Ok(old_token)
    }

    fn expected_but(&self, kind: &str) -> Error {
        Error::parse(
            format!("expected {}, but got {}", kind, display(self.current_token)),
            self.current_token.span.start,
            self.current_token.span.line,
        )
    }

    fn expect(&mut self, kind: TokenKind) -> Result<Token<'a>> {
        if !self.current_is(kind) {
            return Err(self.expected_but(&format!("{kind:?}")));
        }
        return self.consume();
    }

    fn parse_expression(&mut self) -> Result<Expression<'a>> {}

    fn parse_assign_node(&mut self) -> Result<Statement<'a>> {
        let start = self.current_token.span;

        let identifier = self.expect(TokenKind::Identifier)?;
        let equals = self.expect(TokenKind::Equals)?;
        let value = self.parse_expression()?;

        Ok(Statement::Assign(Assign {
            identifier,
            equals,
            value: value.clone(),
            span: Span {
                start: start.start,
                end: value.span().end,
                line: start.line,
            },
        }))
    }

    fn parse_block_node(&mut self) -> Result<Block<'a>> {
        let start = self.current_token.span;
        let mut body = vec![];

        while self.current_kind != TokenKind::Eof {
            if self.lookahead_is(TokenKind::Equals) {
                body.push(self.parse_assign_node()?);
            }
        }

        Ok(Block {
            body,
            span: Span {
                start: start.start,
                end: self.current_token.span.end,
                line: start.line,
            },
        })
    }
}
