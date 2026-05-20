use crate::{
    ast::{
        Lexer, Token, TokenKind,
        ast::{
            Assign, Ast, Block, Delimited, Expression, ExpressionTable, LetStatement, Route,
            RouteKind, Separate, Separated, ServiceTarget, SimpleExpression, Span, Statement,
            TableField, TableFieldNameKey, TableFieldNoKey, VarRoot,
        },
    },
    error::{Error, Result},
};

pub struct Parser<'a> {
    pub lexer: Lexer<'a>,
    pub current_token: Token<'a>,
    pub lookahead_token: Token<'a>,
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
            | Expression::String(node) => node.span,
            Expression::Table(node) => node.span,
        }
    }
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a [u8]) -> Result<Self> {
        let mut lexer = Lexer::new(source);
        let current_token = lexer.next_token()?;
        let lookahead_token = lexer.next_token()?;
        Ok(Self {
            lexer,
            current_token,
            lookahead_token,
        })
    }

    fn consume(&mut self) -> Result<Token<'a>> {
        let old_token = self.current_token;
        self.current_token = self.lookahead_token;
        self.lookahead_token = self.lexer.next_token()?;
        Ok(old_token)
    }

    fn current_is(&self, kind: TokenKind) -> bool {
        return self.current_token.kind == kind;
    }

    fn lookahead_is(&self, kind: TokenKind) -> bool {
        return self.lookahead_token.kind == kind;
    }

    fn is_delimiter(&self) -> bool {
        return self.current_is(TokenKind::LBrace)
            || self.current_is(TokenKind::RBrace)
            || self.current_is(TokenKind::Newline)
            || self.current_is(TokenKind::Eof);
    }

    fn expected_but(&self, kind: &str) -> Error {
        return Error::parse(
            format!(
                "expected {}, but got {:?}",
                kind,
                display(self.current_token)
            ),
            self.current_token.span.line,
            self.current_token.span.start,
        );
    }

    fn expect(&mut self, kind: TokenKind) -> Result<Token<'a>> {
        if self.current_is(kind) {
            return self.consume();
        } else {
            return Err(self.expected_but(&format!("{kind:?}")));
        }
    }

    fn skip_current(&mut self) -> Result<()> {
        while self.current_is(TokenKind::Whitespace) {
            self.consume()?;
        }
        Ok(())
    }

    pub fn parse(&mut self) -> Result<Ast<'a>> {
        let block = self.parse_block_node()?;
        Ok(Ast { block })
    }

    fn parse_block_node(&mut self) -> Result<Block<'a>> {
        let start = self.current_token.span;

        let mut body = vec![];
        while self.current_token.kind != TokenKind::Eof {
            self.skip_current()?;

            if self.current_is(TokenKind::Tcp) {
                println!("parsing tcp node");
                body.push(self.parse_tcp_node()?)
            } else if self.current_is(TokenKind::Let) {
                body.push(self.parse_var_node()?);
            } else if self.lookahead_is(TokenKind::Equals) {
                body.push(self.parse_assign_node()?);
            } else if self.current_is(TokenKind::Identifier) {
                body.push(self.parse_route_node()?);
            } else {
                return Err(self.expected_but("statement"));
            }
        }

        let end = self.current_token.span;

        Ok(Block {
            body,
            span: Span {
                start: start.start,
                end: end.end,
                line: start.line,
            },
        })
    }

    fn parse_route_node(&mut self) -> Result<Statement<'a>> {
        let start = self.current_token.span;
        let fqdn = self.expect(TokenKind::Identifier)?;
        self.expect(TokenKind::Arrow)?;
        let service_target = self.parse_service_node()?;

        let block_start = self.current_token.span;
        self.expect(TokenKind::LBrace)?;

        let mut properties = vec![];
        while self.current_token.kind != TokenKind::RBrace
            && self.current_token.kind != TokenKind::Eof
        {
            properties.push(self.parse_assign_node()?);
        }
        let end = self.current_token.span;
        self.expect(TokenKind::RBrace)?;

        Ok(Statement::Route(Route {
            kind: RouteKind::HTTP,
            hostname: fqdn,
            target: service_target,
            properties: Block {
                body: properties,
                span: Span {
                    start: block_start.start,
                    end: end.end,
                    line: block_start.line,
                },
            },
            span: Span {
                start: start.start,
                end: end.end,
                line: start.line,
            },
        }))
    }

    fn parse_var_node(&mut self) -> Result<Statement<'a>> {
        let start = self.current_token.span;
        let var = self.expect(TokenKind::Let)?;
        let name = self.expect(TokenKind::Identifier)?;
        self.expect(TokenKind::Equals)?;
        let value = self.parse_expression()?;
        let end = self.current_token.span;

        Ok(Statement::Var(LetStatement {
            root: VarRoot {
                var,
                name,
                span: Span {
                    start: start.start,
                    end: name.span.end,
                    line: start.line,
                },
            },
            value,
            span: Span {
                start: start.start,
                end: end.end,
                line: start.line,
            },
        }))
    }

    fn parse_tcp_node(&mut self) -> Result<Statement<'a>> {
        let start = self.current_token.span;

        self.expect(TokenKind::Tcp)?;

        let service_target = self.parse_service_node()?;
        let block_start = self.current_token.span;
        self.expect(TokenKind::LBrace)?;

        let mut properties = vec![];
        while self.current_token.kind != TokenKind::RBrace
            && self.current_token.kind != TokenKind::Eof
        {
            properties.push(self.parse_assign_node()?)
        }
        let end = self.current_token.span;
        self.expect(TokenKind::RBrace)?;

        Ok(Statement::Route(Route {
            kind: RouteKind::TCP,
            hostname: service_target.service,
            target: service_target,
            properties: Block {
                body: properties,
                span: Span {
                    start: block_start.start,
                    end: end.end,
                    line: block_start.line,
                },
            },
            span: Span {
                start: start.start,
                end: end.end,
                line: start.line,
            },
        }))
    }

    fn parse_service_node(&mut self) -> Result<ServiceTarget<'a>> {
        let start = self.current_token.span;
        let service = self.expect(TokenKind::Identifier)?;
        self.expect(TokenKind::Colon)?;
        let port_token = self.expect(TokenKind::Number)?;
        let port = port_token.text.parse::<usize>().map_err(|e| {
            Error::parse(
                format!("invalid number: {e:?}"),
                port_token.span.line,
                port_token.span.start,
            )
        })?;

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

    fn parse_assign_node(&mut self) -> Result<Statement<'a>> {
        let start = self.current_token.span;
        let identifier = self.expect(TokenKind::Identifier)?;
        let equals = self.expect(TokenKind::Equals)?;
        let expression = self.parse_expression()?;
        let end = self.current_token.span;

        Ok(Statement::Assign(Assign {
            identifier,
            equals,
            value: expression,
            span: Span {
                start: start.start,
                end: end.end,
                line: start.line,
            },
        }))
    }

    fn parse_tablefield_nokey(&mut self) -> Result<TableFieldNoKey<'a>> {
        let expression = self.parse_expression()?;

        return Ok(TableFieldNoKey {
            value: expression.clone(),
            span: expression.span(),
        });
    }

    fn parse_delimiter<V>(
        &mut self,
        left: TokenKind,
        right: TokenKind,
        call: fn() -> V,
    ) -> Result<Delimited<'a, V>> {
        let token_left = self.expect(left)?;
        let value = call();
        let token_right = self.expect(right)?;

        Ok(Delimited {
            left: token_left,
            right: token_right,
            value,
        })
    }

    fn separated<T>(&mut self, call: fn() -> T) -> Result<Separated<'a, T>> {
        let mut values = Vec::new();

        while !self.is_delimiter() {
            let current_pos = self.current_token.span;

            let value = call();
            let mut separator: Option<Token<'a>> = None;
            if self.current_is(TokenKind::Comma) {
                separator = Some(self.expect(TokenKind::Comma)?)
            }

            let end;
            if let Some(separator) = separator {
                end = separator.span.end
            } else {
                end = current_pos.end
            }

            values.push(Separate {
                value,
                separator,
                span: Span {
                    start: current_pos.start,
                    end,
                    line: current_pos.line,
                },
            });
        }

        Ok(values)
    }

    fn parse_tablefield_namekey(&mut self) -> Result<TableFieldNameKey<'a>> {
        let name = self.expect(TokenKind::Identifier)?;
        let equals = self.expect(TokenKind::Equals)?;
        let value = self.parse_expression()?;

        Ok(TableFieldNameKey {
            name,
            equals,
            value: value.clone(),
            span: Span {
                start: name.span.start,
                end: value.span().end,
                line: name.span.line,
            },
        })
    }

    fn parse_tablefield(&mut self) -> Result<TableField<'a>> {
        if self.current_is(TokenKind::Identifier) && self.lookahead_is(TokenKind::Equals) {
            Ok(TableField::NameKey(self.parse_tablefield_namekey()?))
        } else {
            Ok(TableField::NoKey(self.parse_tablefield_nokey()?))
        }
    }

    fn parse_table(&mut self) -> Result<ExpressionTable<'a>> {
        let left = self.expect(TokenKind::LBrace)?;

        let mut fields = Vec::new();
        while !self.is_delimiter() {
            let current_pos = self.current_token.span;
            let field = self.parse_tablefield()?;

            let separator = if self.current_is(TokenKind::Comma) {
                Some(self.expect(TokenKind::Comma)?)
            } else {
                None
            };

            let end = separator.map(|s| s.span.end).unwrap_or(current_pos.end);

            fields.push(Separate {
                value: field,
                separator,
                span: Span {
                    start: current_pos.start,
                    end,
                    line: current_pos.line,
                },
            });
        }

        let right = self.expect(TokenKind::RBrace)?;

        Ok(ExpressionTable {
            values: Delimited {
                left,
                value: fields,
                right,
            },
            span: Span {
                start: left.span.start,
                end: right.span.end,
                line: left.span.line,
            },
        })
    }

    fn parse_expression(&mut self) -> Result<Expression<'a>> {
        match self.current_token.kind {
            TokenKind::Nil => {
                let token = self.expect(TokenKind::Nil)?;
                Ok(Expression::Nil(SimpleExpression {
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
            TokenKind::Identifier => {
                let token = self.expect(TokenKind::Identifier)?;
                Ok(Expression::String(SimpleExpression {
                    token,
                    span: token.span,
                }))
            }
            TokenKind::LBrace => {
                let expression = self.parse_table()?;
                Ok(Expression::Table(expression))
            }
            kind => {
                if kind == TokenKind::True || kind == TokenKind::False {
                    let token = self.consume()?;
                    Ok(Expression::Boolean(SimpleExpression {
                        token,
                        span: token.span,
                    }))
                } else {
                    return Err(self.expected_but(&display(self.current_token)));
                }
            }
        }
    }
}
