use crate::{
    ast::ast::{
        Assign, Ast, Block, Delimited, Expression, ExpressionTable, LetStatement, Route, RouteHTTP,
        RouteTCP, Separate, ServiceTarget, SimpleExpression, Span, Statement, TableField,
        TableFieldNameKey, TableFieldNoKey, Token, TokenKind, VarRoot,
    },
    error::{Error, Result},
};

struct Lexer<'a> {
    input: &'a [u8],
    pos: usize,
    len: usize,
    line: usize,
}

pub struct Parser<'a> {
    pub lexer: Lexer<'a>,
    pub current_token: Token<'a>,
    pub current_kind: TokenKind,
    pub lookahead_token: Token<'a>,
    pub lookahead_kind: TokenKind,
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
                    self.bump();
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
            b',' => {
                self.bump();
                TokenKind::Comma
            }
            mut c if c.is_ascii_digit() => {
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
                    "route" => TokenKind::Route,
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

        while kind == TokenKind::Whitespace {
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

fn is_delimiter(kind: TokenKind) -> bool {
    return kind == TokenKind::LBrace
        || kind == TokenKind::RBrace
        || kind == TokenKind::Newline
        || kind == TokenKind::Eof;
}

impl<'a> Expression<'a> {
    pub fn span(&self) -> Span {
        match self {
            Expression::Boolean(node)
            | Expression::Nil(node)
            | Expression::Number(node)
            | Expression::String(node) => node.span,
            Expression::Table(node) => node.span,
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
            self.current_token.span.line,
            self.current_token.span.start,
        )
    }

    fn expect(&mut self, kind: TokenKind) -> Result<Token<'a>> {
        if !self.current_is(kind) {
            return Err(self.expected_but(&format!("{kind:?}")));
        }
        return self.consume();
    }

    fn parse_tablefield_namekey(&mut self) -> Result<TableFieldNameKey<'a>> {
        let start = self.current_token.span;

        let name = self.expect(TokenKind::Identifier)?;
        let equals = self.expect(TokenKind::Equals)?;
        let value = self.parse_expression()?;

        Ok(TableFieldNameKey {
            name,
            equals,
            value: value.clone(),
            span: Span {
                start: start.start,
                end: value.span().end,
                line: start.line,
            },
        })
    }

    fn parse_tablefield_nokey(&mut self) -> Result<TableFieldNoKey<'a>> {
        let value = self.parse_expression()?;

        Ok(TableFieldNoKey {
            value: value.clone(),
            span: value.span(),
        })
    }

    fn parse_tablefield(&mut self) -> Result<TableField<'a>> {
        if self.current_is(TokenKind::Identifier) && self.lookahead_is(TokenKind::Equals) {
            Ok(TableField::NameKey(self.parse_tablefield_namekey()?))
        } else {
            Ok(TableField::NoKey(self.parse_tablefield_nokey()?))
        }
    }

    fn parse_table(&mut self) -> Result<Expression<'a>> {
        let start = self.current_token.span;
        let left = self.expect(TokenKind::LBrace)?;

        let mut values = vec![];
        while !is_delimiter(self.current_kind) {
            let start = self.current_token.span;

            let value = self.parse_tablefield()?;

            let separator = if self.current_is(TokenKind::Comma) {
                Some(self.expect(TokenKind::Comma)?)
            } else {
                None
            };

            let end = separator.map(|s| s.span.end).unwrap_or(start.end);

            values.push(Separate {
                value,
                separator,
                span: Span {
                    start: start.start,
                    end,
                    line: start.line,
                },
            });
        }

        let right = self.expect(TokenKind::RBrace)?;

        Ok(Expression::Table(ExpressionTable {
            values: Delimited {
                left,
                right,
                value: values,
            },
            span: Span {
                start: start.start,
                end: right.span.end,
                line: start.line,
            },
        }))
    }

    fn parse_expression(&mut self) -> Result<Expression<'a>> {
        match self.current_kind {
            TokenKind::LBrace => {
                let expression = self.parse_table()?;
                Ok(expression)
            }
            TokenKind::Identifier => {
                let token = self.expect(TokenKind::Identifier)?;
                Ok(Expression::String(SimpleExpression {
                    token,
                    span: token.span,
                }))
            }
            TokenKind::Number => {
                let token = self.expect(TokenKind::Number)?;
                Ok(Expression::Number(SimpleExpression {
                    token,
                    span: token.span,
                }))
            }
            TokenKind::Nil => {
                let token = self.expect(TokenKind::Nil)?;
                Ok(Expression::Nil(SimpleExpression {
                    token,
                    span: token.span,
                }))
            }
            kind => {
                if kind == TokenKind::True || kind == TokenKind::False {
                    let token = self.consume()?;
                    Ok(Expression::Boolean(SimpleExpression {
                        token,
                        span: token.span,
                    }))
                } else {
                    return Err(self.expected_but("expression"));
                }
            }
        }
    }

    fn parse_var_root(&mut self) -> Result<VarRoot<'a>> {
        let start = self.current_token.span;

        let var = self.expect(TokenKind::Let)?;
        let name = self.expect(TokenKind::Identifier)?;

        Ok(VarRoot {
            var,
            name,
            span: Span {
                start: start.start,
                end: name.span.end,
                line: start.line,
            },
        })
    }

    fn parse_var_node(&mut self) -> Result<Statement<'a>> {
        let start = self.current_token.span;

        let root = self.parse_var_root()?;
        self.expect(TokenKind::Equals)?;
        let value = self.parse_expression()?;

        Ok(Statement::Var(LetStatement {
            root,
            value: value.clone(),
            span: Span {
                start: start.start,
                end: value.span().end,
                line: start.line,
            },
        }))
    }

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

    fn parse_service_target(&mut self) -> Result<ServiceTarget<'a>> {
        let start = self.current_token.span;

        let service = self.expect(TokenKind::Identifier)?;
        self.expect(TokenKind::Colon)?;
        let port_token = self.expect(TokenKind::Number)?;

        let port = match port_token.text.parse::<usize>() {
            Ok(port) => port,
            Err(_) => {
                return Err(Error::parse(
                    format!(
                        "provided invalid number '{}', only integers are supported",
                        port_token.text
                    ),
                    start.line,
                    start.start,
                ));
            }
        };

        Ok(ServiceTarget {
            service,
            port,
            span: Span {
                start: start.start,
                end: port_token.span.end,
                line: start.line,
            },
        })
    }

    fn parse_route_properties(&mut self) -> Result<Block<'a>> {
        let start = self.expect(TokenKind::LBrace)?.span;
        let mut body = vec![];

        while !self.current_is(TokenKind::RBrace) && !self.current_is(TokenKind::Eof) {
            body.push(self.parse_assign_node()?);
        }

        let end = self.expect(TokenKind::RBrace)?.span.end;

        Ok(Block {
            body,
            span: Span {
                start: start.start,
                end,
                line: start.line,
            },
        })
    }

    fn parse_route_tcp(&mut self) -> Result<RouteTCP<'a>> {
        let start = self.current_token.span;
        self.expect(TokenKind::Tcp)?;

        let target = self.parse_service_target()?;
        let properties = self.parse_route_properties()?;

        Ok(RouteTCP {
            target,
            properties: properties.clone(),
            span: Span {
                start: start.start,
                end: properties.span.end,
                line: start.line,
            },
        })
    }

    fn parse_route_http(&mut self) -> Result<RouteHTTP<'a>> {
        let start = self.current_token.span;

        let hostname = self.expect(TokenKind::Identifier)?;
        self.expect(TokenKind::Arrow)?;
        let target = self.parse_service_target()?;
        let properties = self.parse_route_properties()?;

        Ok(RouteHTTP {
            hostname,
            target,
            properties: properties.clone(),
            span: Span {
                start: start.start,
                end: properties.span.end,
                line: start.line,
            },
        })
    }

    fn parse_route(&mut self) -> Result<Statement<'a>> {
        self.expect(TokenKind::Route)?;

        if self.current_is(TokenKind::Tcp) {
            Ok(Statement::Route(Route::TCP(self.parse_route_tcp()?)))
        } else {
            Ok(Statement::Route(Route::HTTP(self.parse_route_http()?)))
        }
    }

    fn parse_block_node(&mut self) -> Result<Block<'a>> {
        let start = self.current_token.span;
        let mut body = vec![];

        while self.current_kind != TokenKind::Eof {
            if self.lookahead_is(TokenKind::Equals) {
                body.push(self.parse_assign_node()?);
            } else if self.current_is(TokenKind::Let) {
                body.push(self.parse_var_node()?);
            } else if self.current_is(TokenKind::Route) {
                body.push(self.parse_route()?);
            } else {
                return Err(self.expected_but("statement"));
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
