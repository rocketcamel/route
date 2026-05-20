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
    Comma,
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

#[derive(Debug, Clone)]
pub struct Delimited<'a, VALUE> {
    pub left: Token<'a>,
    pub value: VALUE,
    pub right: Token<'a>,
}

pub type Separated<'a, T> = Vec<Separate<'a, T>>;

#[derive(Debug, Clone)]
pub struct Separate<'a, T> {
    pub value: T,
    pub separator: Option<Token<'a>>,
    pub span: Span,
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
pub struct VarRoot<'a> {
    pub var: Token<'a>,
    pub name: Token<'a>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct TableFieldNameKey<'a> {
    pub name: Token<'a>,
    pub equals: Token<'a>,
    pub value: Expression<'a>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct TableFieldNoKey<'a> {
    pub value: Expression<'a>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum TableField<'a> {
    NameKey(TableFieldNameKey<'a>),
    NoKey(TableFieldNoKey<'a>),
}

#[derive(Debug, Clone)]
pub struct ExpressionTable<'a> {
    pub values: Delimited<'a, Separated<'a, TableField<'a>>>,
    pub span: Span,
}

#[derive(Debug)]
pub struct LetStatement<'a> {
    pub root: VarRoot<'a>,
    pub value: Expression<'a>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct SimpleExpression<'a> {
    pub token: Token<'a>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Expression<'a> {
    Boolean(SimpleExpression<'a>),
    Nil(SimpleExpression<'a>),
    Number(SimpleExpression<'a>),
    String(SimpleExpression<'a>),
    Table(ExpressionTable<'a>),
}

#[derive(Debug)]
pub struct Assign<'a> {
    pub identifier: Token<'a>,
    pub equals: Token<'a>,
    pub value: Expression<'a>,
    pub span: Span,
}

#[derive(Debug)]
pub enum Statement<'a> {
    Assign(Assign<'a>),
    Var(LetStatement<'a>),
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
