use crate::{
    ast::ast::{
        Assign, Ast, BinaryOperator, Block, Delimited, Expression, ExpressionBinary,
        ExpressionTable, ExpressionUnary, LetStatement, Route, RouteHTTP, RouteTCP, Separate,
        ServiceTarget, SimpleExpression, Span, Statement, TableField, TableFieldNameKey,
        TableFieldNoKey, Token, TokenKind, UnaryOperator, VarRoot,
    },
    error::{Error, Result},
};

pub struct Lexer<'a> {
    input: &'a [u8],
    pos: usize,
    len: usize,
    line: usize,
    col: usize,
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
            col: 1,
        }
    }

    fn peek(&mut self) -> u8 {
        if self.pos == self.len {
            return 0;
        }
        self.input[self.pos]
    }

    fn bump(&mut self) {
        let c = self.input[self.pos];
        self.pos = self.len.min(self.pos + 1);

        if c == b'\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1
        }
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
                    TokenKind::Subtract
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
                let c = self.bump_peek();

                if c == b'=' {
                    TokenKind::BinaryEquals
                } else {
                    TokenKind::Equals
                }
            }
            b'!' => {
                let c = self.bump_peek();

                if c == b'=' {
                    TokenKind::NEquals
                } else {
                    TokenKind::Not
                }
            }
            b'>' => {
                let c = self.bump_peek();

                if c == b'=' {
                    TokenKind::GreaterEquals
                } else {
                    TokenKind::Greater
                }
            }
            b'<' => {
                let c = self.bump_peek();

                if c == b'=' {
                    TokenKind::LessEquals
                } else {
                    TokenKind::Less
                }
            }
            b'+' => {
                self.bump();
                TokenKind::Add
            }
            b'*' => {
                self.bump();
                TokenKind::Multiply
            }
            b'/' => {
                self.bump();
                TokenKind::Divide
            }
            b'^' => {
                self.bump();
                TokenKind::Exponent
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
                    "and" => TokenKind::And,
                    "or" => TokenKind::Or,
                    "tcp" => TokenKind::Tcp,
                    "let" => TokenKind::Let,
                    "route" => TokenKind::Route,
                    _ => TokenKind::Identifier,
                }
            }
            c if is_whitespace(c) => {
                self.bump();
                TokenKind::Whitespace
            }
            _ => {
                self.bump();
                TokenKind::Error
            }
        }
    }

    fn next_token(&mut self) -> Result<Token<'a>> {
        let mut start = self.pos;
        let mut initial_line = self.line;
        let mut initial_col = self.col;
        let mut kind = self.read_kind();

        while kind == TokenKind::Whitespace {
            start = self.pos;
            initial_line = self.line;
            initial_col = self.col;
            kind = self.read_kind();
        }

        let span = Span {
            x: start,
            y: self.pos,
            z: initial_line,
            w: initial_col,
        };

        if kind == TokenKind::Error {
            let value = str::from_utf8(&self.input[start..self.pos]).unwrap();
            return Err(Error::parse(
                format!("cannot parse '{value}' into a token"),
                span,
            ));
        }
        let text = str::from_utf8(&self.input[start..self.pos]).unwrap();

        Ok(Token { kind, text, span })
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
            Expression::Binary(node) => node.span,
            Expression::Unary(node) => node.span,
            Expression::Table(node) => node.span,
        }
    }
}

impl<'a> TableField<'a> {
    pub fn span(&self) -> Span {
        match self {
            TableField::NameKey(n) => n.span,
            TableField::NoKey(n) => n.span,
        }
    }
}

fn to_span(spans: &[Option<Span>]) -> Span {
    let first = spans.iter().flatten().next().copied();
    let last = spans.iter().flatten().next_back().copied();

    let x = first.map(|v| v.x).unwrap_or(0);
    let y = last.map(|v| v.y).unwrap_or(0);
    let z = first.map(|v| v.z).unwrap_or(0);
    let w = first.map(|v| v.w).unwrap_or(0);

    Span { x, y, z, w }
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
            self.current_token.span,
        )
    }

    fn expect(&mut self, kind: TokenKind) -> Result<Token<'a>> {
        if !self.current_is(kind) {
            return Err(self.expected_but(&format!("{kind:?}")));
        }
        return self.consume();
    }

    fn parse_tablefield_namekey(&mut self) -> Result<TableFieldNameKey<'a>> {
        let name = self.expect(TokenKind::Identifier)?;
        let equals = self.expect(TokenKind::Equals)?;
        let value = self.parse_expression(None)?;

        Ok(TableFieldNameKey {
            name,
            equals,
            value: value.clone(),
            span: to_span(&[Some(name.span), Some(equals.span), Some(value.span())]),
        })
    }

    fn parse_tablefield_nokey(&mut self) -> Result<TableFieldNoKey<'a>> {
        let value = self.parse_expression(None)?;

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

    fn parse_delimiter<V, F: FnOnce(&mut Self) -> Result<V>>(
        &mut self,
        left: TokenKind,
        right: TokenKind,
        call: F,
    ) -> Result<Delimited<'a, V>> {
        let left = self.expect(left)?;
        let value = call(self)?;
        let right = self.expect(right)?;

        Ok(Delimited { left, value, right })
    }

    fn parse_table(&mut self) -> Result<Expression<'a>> {
        let values = self.parse_delimiter(TokenKind::LBrace, TokenKind::RBrace, |parser| {
            let mut values = Vec::new();

            while !is_delimiter(parser.current_kind) {
                let value = parser.parse_tablefield()?;

                let separator = if parser.current_is(TokenKind::Comma) {
                    Some(parser.expect(TokenKind::Comma)?)
                } else {
                    None
                };

                let separator_span = separator.map(|s| s.span);
                let span = value.span();

                values.push(Separate {
                    value,
                    separator,
                    span: to_span(&[Some(span), separator_span]),
                });
            }

            Ok(values)
        })?;

        let left_span = values.left.span;
        let first_span = values.value.first().map(|v| v.span);
        let last_span = values.value.last().map(|v| v.span);
        let right_span = values.right.span;

        Ok(Expression::Table(ExpressionTable {
            values,
            span: to_span(&[Some(left_span), first_span, last_span, Some(right_span)]),
        }))
    }

    fn current_binary_operator(&self) -> Option<Token<'a>> {
        if self.current_is(TokenKind::BinaryEquals)
            || self.current_is(TokenKind::NEquals)
            || self.current_is(TokenKind::Greater)
            || self.current_is(TokenKind::Less)
            || self.current_is(TokenKind::GreaterEquals)
            || self.current_is(TokenKind::LessEquals)
            || self.current_is(TokenKind::Add)
            || self.current_is(TokenKind::Subtract)
            || self.current_is(TokenKind::Multiply)
            || self.current_is(TokenKind::Divide)
            || self.current_is(TokenKind::Exponent)
            || self.current_is(TokenKind::And)
            || self.current_is(TokenKind::Or)
        {
            Some(self.current_token)
        } else {
            None
        }
    }

    fn binary_op_precedence(&self, operator: BinaryOperator) -> (usize, usize) {
        if operator == BinaryOperator::Add || operator == BinaryOperator::Subtract {
            (6, 6)
        } else if operator == BinaryOperator::Multiply || operator == BinaryOperator::Divide {
            (7, 7)
        } else if operator == BinaryOperator::Exponent {
            (10, 9)
        } else if operator == BinaryOperator::BinaryEquals || operator == BinaryOperator::NEquals {
            (3, 3)
        } else if operator == BinaryOperator::Less
            || operator == BinaryOperator::Greater
            || operator == BinaryOperator::LessEquals
            || operator == BinaryOperator::GreaterEquals
        {
            (3, 3)
        } else if operator == BinaryOperator::And {
            (2, 2)
        } else if operator == BinaryOperator::Or {
            (1, 1)
        } else {
            unreachable!()
        }
    }

    fn parse_simple_expression(&mut self) -> Result<Expression<'a>> {
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
            _ if self.current_binary_operator().is_some() => {
                let token = self.consume()?;
                Err(Error::parse(
                    "missing left hand side of binary operator",
                    token.span,
                ))
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

    fn parse_unary_operator(&mut self) -> Result<Option<Token<'a>>> {
        if self.current_is(TokenKind::Negate) || self.current_is(TokenKind::Not) {
            let result = self.consume()?;

            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    fn parse_expression(&mut self, limit: Option<usize>) -> Result<Expression<'a>> {
        let limit = limit.unwrap_or(0);

        let mut expr: Expression;

        let unary_operator = self.parse_unary_operator()?;

        if let Some(unary_operator) = unary_operator {
            let rhs = self.parse_expression(None)?;
            let kind: UnaryOperator = unary_operator.kind.try_into().unwrap();

            let rhs_span = rhs.span();

            expr = Expression::Unary(ExpressionUnary {
                operator: kind,
                value: rhs.into(),
                span: to_span(&[Some(unary_operator.span), Some(rhs_span)]),
            });
        } else {
            expr = self.parse_simple_expression()?
        }

        loop {
            let Some(binop) = self.current_binary_operator() else {
                break;
            };

            let kind = binop.kind.try_into().unwrap();
            let (left_precedence, right_precedence) = self.binary_op_precedence(kind);

            if left_precedence < limit {
                break;
            }

            self.consume()?;
            let rhs = self.parse_expression(Some(right_precedence))?;

            let lhs_span = expr.span();
            let rhs_span = rhs.span();

            expr = Expression::Binary(ExpressionBinary {
                left: expr.into(),
                operator: kind,
                right: rhs.clone().into(),
                span: to_span(&[Some(lhs_span), Some(binop.span), Some(rhs_span)]),
            })
        }

        Ok(expr)
    }

    fn parse_var_root(&mut self) -> Result<VarRoot<'a>> {
        let var = self.expect(TokenKind::Let)?;
        let name = self.expect(TokenKind::Identifier)?;

        Ok(VarRoot {
            var,
            name,
            span: to_span(&[Some(var.span), Some(name.span)]),
        })
    }

    fn parse_var_node(&mut self) -> Result<Statement<'a>> {
        let root = self.parse_var_root()?;
        let equals = self.expect(TokenKind::Equals)?;
        let value = self.parse_expression(None)?;

        let root_span = root.span;
        let value_span = value.span();

        Ok(Statement::Var(LetStatement {
            root,
            value: value.clone(),
            span: to_span(&[Some(root_span), Some(equals.span), Some(value_span)]),
        }))
    }

    fn parse_assign_node(&mut self) -> Result<Statement<'a>> {
        let identifier = self.expect(TokenKind::Identifier)?;
        let equals = self.expect(TokenKind::Equals)?;
        let value = self.parse_expression(None)?;

        let value_span = value.span();

        Ok(Statement::Assign(Assign {
            identifier,
            equals,
            value: value.clone(),
            span: to_span(&[Some(identifier.span), Some(equals.span), Some(value_span)]),
        }))
    }

    fn parse_service_target(&mut self) -> Result<ServiceTarget<'a>> {
        let service = self.expect(TokenKind::Identifier)?;
        let equals = self.expect(TokenKind::Colon)?;
        let port_token = self.expect(TokenKind::Number)?;

        let port = match port_token.text.parse::<usize>() {
            Ok(port) => port,
            Err(_) => {
                return Err(Error::parse(
                    format!(
                        "provided invalid number '{}', only integers are supported",
                        port_token.text
                    ),
                    service.span,
                ));
            }
        };

        Ok(ServiceTarget {
            service,
            port,
            span: to_span(&[Some(service.span), Some(equals.span), Some(port_token.span)]),
        })
    }

    fn parse_route_properties(&mut self) -> Result<Block<'a>> {
        let left = self.expect(TokenKind::LBrace)?;

        let mut body = vec![];

        while !self.current_is(TokenKind::RBrace) && !self.current_is(TokenKind::Eof) {
            body.push(self.parse_assign_node()?);
        }

        let right = self.expect(TokenKind::RBrace)?;

        Ok(Block {
            body,
            span: to_span(&[Some(left.span), Some(right.span)]),
        })
    }

    fn parse_route_tcp(&mut self) -> Result<RouteTCP<'a>> {
        let start = self.expect(TokenKind::Tcp)?;

        let target = self.parse_service_target()?;
        let properties = self.parse_route_properties()?;

        let target_span = target.span;
        let properties_span = properties.span;

        Ok(RouteTCP {
            target,
            properties: properties.clone(),
            span: to_span(&[Some(start.span), Some(target_span), Some(properties_span)]),
        })
    }

    fn parse_route_http(&mut self) -> Result<RouteHTTP<'a>> {
        let hostname = self.expect(TokenKind::Identifier)?;
        let equals = self.expect(TokenKind::Arrow)?;
        let target = self.parse_service_target()?;
        let properties = self.parse_route_properties()?;

        let target_span = target.span;
        let properties_span = properties.span;

        Ok(RouteHTTP {
            hostname,
            target,
            properties: properties.clone(),
            span: to_span(&[
                Some(hostname.span),
                Some(equals.span),
                Some(target_span),
                Some(properties_span),
            ]),
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
            if self.lookahead_is(TokenKind::BinaryEquals) {
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
                x: start.x,
                y: self.current_token.span.y,
                z: start.z,
                w: start.w,
            },
        })
    }
}
