#[derive(Debug, Clone, Copy)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub text: &'a str,
    pub span: Span,
}

#[allow(unused)]
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum TokenKind {
    // symbols
    Arrow,
    LBrace,
    RBrace,
    Colon,
    Identifier,
    Number,
    Equals,
    // keywords
    True,
    False,
    Nil,
    Tcp,
    Let,
    // whitespace
    Whitespace,
    Comment,
    // line endings
    Eof,
    Newline,
    Error,
}

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
}

#[derive(Debug)]
pub struct ServiceTarget<'a> {
    pub service: Token<'a>,
    pub port: usize,
    pub span: Span,
}

#[derive(Debug)]
pub enum RouteKind {
    HTTP,
    TCP,
}

#[derive(Debug)]
pub struct Route<'a> {
    pub kind: RouteKind,
    pub hostname: Token<'a>,
    pub target: ServiceTarget<'a>,
    pub properties: Block<'a>,
    pub span: Span,
}

#[derive(Debug)]
pub struct SimpleExpression<'a> {
    pub token: Token<'a>,
    pub span: Span,
}

#[derive(Debug)]
pub enum Expression<'a> {
    Boolean(SimpleExpression<'a>),
    Nil(SimpleExpression<'a>),
    Number(SimpleExpression<'a>),
    String(SimpleExpression<'a>),
}

#[derive(Debug)]
pub struct Assign<'a> {
    pub identifier: Token<'a>,
    pub equals: Token<'a>,
    pub value: Expression<'a>,
}

#[derive(Debug)]
pub enum Statement<'a> {
    Assign(Assign<'a>),
    Route(Route<'a>),
}

#[derive(Debug)]
pub struct Block<'a> {
    pub body: Vec<Statement<'a>>,
    pub span: Span,
}

#[derive(Debug)]
pub struct Ast<'a> {
    pub block: Block<'a>,
}
