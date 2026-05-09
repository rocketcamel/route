use crate::{
    ast::{
        Kw, Lexer, Token, TokenKind,
        ast::{Ast, Block, Gateway, Property, PropertyKind, Route, ServiceTarget, Span, Statement},
    },
    error::{Error, Result},
};

pub struct Parser<'a> {
    pub lexer: Lexer<'a>,
    pub current_token: Token<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a [u8]) -> Result<Self> {
        let mut lexer = Lexer::new(source);
        let current_token = lexer.next_token()?;
        Ok(Self {
            lexer,
            current_token,
        })
    }

    fn consume(&mut self) -> Result<Token<'a>> {
        let old_token = self.current_token;
        let token = self.lexer.next_token()?;
        self.current_token = token;
        Ok(old_token)
    }

    fn current_is(&self, kind: TokenKind) -> bool {
        return self.current_token.kind == kind;
    }

    fn expect(&mut self, kind: TokenKind) -> Result<Token<'a>> {
        if self.current_is(kind) {
            return self.consume();
        } else {
            Err(Error::parse(
                format!("expected {:?}, got {:?}", kind, self.current_token.kind),
                self.current_token.span.line,
                self.current_token.span.start,
            ))
        }
    }

    pub fn parse(&mut self) -> Result<Ast<'a>> {
        let mut statements = vec![];

        while !self.current_is(TokenKind::Eof) {
            statements.push(self.parse_statement_node()?);
        }

        Ok(Ast {
            block: Block {
                body: statements,
                span: Span {
                    start: 0,
                    end: self.lexer.len,
                    line: 0,
                },
            },
        })
    }

    fn parse_statement_node(&mut self) -> Result<Statement<'a>> {
        match self.current_token.kind {
            TokenKind::Keyword(Kw::Gateway) => self.parse_gateway_node(),
            TokenKind::Keyword(Kw::Namespace) => self.parse_property(),
            TokenKind::Identifier => self.parse_route(),
            _ => Err(Error::parse(
                format!("unexpected token {:?}", self.current_token.kind),
                self.current_token.span.line,
                self.current_token.span.start,
            )),
        }
    }

    fn parse_route(&mut self) -> Result<Statement<'a>> {
        let url = self.expect(TokenKind::Identifier)?;
        let start = url.span;

        self.expect(TokenKind::Arrow)?;

        let service_target = self.parse_service_target()?;
        let properties = self.parse_block()?;
        let end = properties.span.end;

        Ok(Statement::Route(Route {
            hostname: url,
            properties,
            target: service_target,
            span: Span {
                start: start.start,
                end,
                line: start.line,
            },
        }))
    }

    fn parse_block(&mut self) -> Result<Block<'a>> {
        let start = self.current_token.span;
        self.expect(TokenKind::LBrace)?;

        let mut body = vec![];
        while self.current_token.kind != TokenKind::RBrace
            && self.current_token.kind != TokenKind::Eof
        {
            body.push(self.parse_property()?);
        }

        let end = self.current_token.span;
        self.expect(TokenKind::RBrace)?;

        Ok(Block {
            body,
            span: Span {
                start: start.start,
                end: end.end,
                line: start.line,
            },
        })
    }

    fn parse_service_target(&mut self) -> Result<ServiceTarget<'a>> {
        let token = self.expect(TokenKind::Identifier)?;
        let start = token.span;
        println!("{token:?}");

        self.expect(TokenKind::Colon)?;

        let port_token = self.expect(TokenKind::Number)?;
        let port = port_token.text.parse::<usize>().map_err(|_| {
            Error::parse(
                format!("invalid port number: {}", port_token.text),
                port_token.span.line,
                port_token.span.start,
            )
        })?;

        Ok(ServiceTarget {
            service: token,
            port,
            span: Span {
                start: start.start,
                end: port_token.span.end,
                line: start.line,
            },
        })
    }

    fn parse_property(&mut self) -> Result<Statement<'a>> {
        let start = self.current_token.span;

        let kind = match self.current_token.kind {
            TokenKind::Keyword(Kw::Namespace) => PropertyKind::Namespace,
            _ => {
                return Err(Error::parse(
                    format!("unexpected property {:?}", self.current_token.text),
                    start.line,
                    start.start,
                ));
            }
        };

        self.consume()?;
        let token = self.expect(TokenKind::Identifier)?;

        Ok(Statement::Property(Property {
            kind,
            token,
            span: Span {
                start: start.start,
                end: token.span.end,
                line: start.line,
            },
        }))
    }

    fn parse_gateway_node(&mut self) -> Result<Statement<'a>> {
        let start = self.current_token.span;
        self.consume()?;
        let token = self.expect(TokenKind::Identifier)?;

        Ok(Statement::Gateway(Gateway {
            token,
            span: Span {
                start: start.start,
                end: token.span.end,
                line: start.line,
            },
        }))
    }
}
