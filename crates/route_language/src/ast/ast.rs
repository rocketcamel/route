use std::fmt::{Binary, Display};

#[derive(Debug, Clone, Copy)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub text: &'a str,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOperator {
    // operators
    BinaryEquals,
    NEquals,
    Greater,
    Less,
    GreaterEquals,
    LessEquals,
    // arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    Exponent,
    // ternary
    And,
    Or,
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryOperator {
    Negate,
    Not,
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

    // binary
    // operators
    BinaryEquals,
    NEquals,
    Greater,
    Less,
    GreaterEquals,
    LessEquals,
    // arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    Exponent,
    // ternary
    And,
    Or,

    // unary
    Negate,
    Not,

    // keywords
    True,
    False,
    Nil,
    Tcp,
    Let,
    Route,
    // whitespace
    Whitespace,
    Comment,
    // line endings
    Eof,
    Newline,
    Error,
}

impl TryFrom<TokenKind> for UnaryOperator {
    type Error = &'static str;

    fn try_from(value: TokenKind) -> Result<Self, Self::Error> {
        if value == TokenKind::Negate {
            Ok(UnaryOperator::Negate)
        } else if value == TokenKind::Not {
            Ok(UnaryOperator::Not)
        } else {
            Err("invalid unary operator")
        }
    }
}

impl TryFrom<TokenKind> for BinaryOperator {
    type Error = &'static str;

    fn try_from(value: TokenKind) -> Result<Self, Self::Error> {
        match value {
            TokenKind::BinaryEquals => Ok(BinaryOperator::BinaryEquals),
            TokenKind::NEquals => Ok(BinaryOperator::NEquals),
            TokenKind::Greater => Ok(BinaryOperator::Greater),
            TokenKind::Less => Ok(BinaryOperator::Less),
            TokenKind::GreaterEquals => Ok(BinaryOperator::GreaterEquals),
            TokenKind::LessEquals => Ok(BinaryOperator::LessEquals),
            TokenKind::Add => Ok(BinaryOperator::Add),
            TokenKind::Subtract => Ok(BinaryOperator::Subtract),
            TokenKind::Multiply => Ok(BinaryOperator::Multiply),
            TokenKind::Divide => Ok(BinaryOperator::Divide),
            TokenKind::Exponent => Ok(BinaryOperator::Exponent),
            TokenKind::And => Ok(BinaryOperator::And),
            TokenKind::Or => Ok(BinaryOperator::Or),
            _ => Err("invalid binary operator"),
        }
    }
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            TokenKind::Arrow => "->",
            TokenKind::LBrace => "{",
            TokenKind::RBrace => "}",
            TokenKind::Colon => ":",
            TokenKind::Identifier => "identifier",
            TokenKind::Number => "number",
            TokenKind::Add => "+",
            TokenKind::Subtract => "-",
            TokenKind::Multiply => "*",
            TokenKind::Divide => "/",
            TokenKind::Exponent => "^",
            TokenKind::Comma => ",",
            TokenKind::Equals => "=",

            TokenKind::BinaryEquals => "==",
            TokenKind::NEquals => "!=",
            TokenKind::Greater => ">",
            TokenKind::Less => "<",
            TokenKind::GreaterEquals => ">=",
            TokenKind::LessEquals => "<=",
            TokenKind::And => "and",
            TokenKind::Or => "or",

            TokenKind::Negate => "-",
            TokenKind::Not => "!",

            TokenKind::True => "true",
            TokenKind::False => "false",
            TokenKind::Nil => "nil",
            TokenKind::Tcp => "tcp",
            TokenKind::Let => "let",
            TokenKind::Route => "route",

            TokenKind::Whitespace => "whitespace",
            TokenKind::Comment => "comment",

            TokenKind::Eof => "eof",
            TokenKind::Newline => "\n",
            TokenKind::Error => "error",
        };

        write!(f, "{text}")
    }
}

impl<'a> Display for Token<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind = self.kind;

        if kind == TokenKind::Identifier {
            write!(f, "{}", self.text)
        } else if kind == TokenKind::Error {
            write!(f, "error {}", self.text)
        } else {
            write!(f, "{kind}")
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub col: usize,
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

#[derive(Debug, Clone)]
pub struct ServiceTarget<'a> {
    pub service: Token<'a>,
    pub port: usize,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct RouteTCP<'a> {
    pub target: ServiceTarget<'a>,
    pub properties: Block<'a>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct RouteHTTP<'a> {
    pub hostname: Token<'a>,
    pub target: ServiceTarget<'a>,
    pub properties: Block<'a>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Route<'a> {
    HTTP(RouteHTTP<'a>),
    TCP(RouteTCP<'a>),
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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
pub struct ExpressionBinary<'a> {
    pub left: Box<Expression<'a>>,
    pub operator: BinaryOperator,
    pub right: Box<Expression<'a>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ExpressionUnary<'a> {
    pub operator: UnaryOperator,
    pub value: Box<Expression<'a>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Expression<'a> {
    Boolean(SimpleExpression<'a>),
    Nil(SimpleExpression<'a>),
    Number(SimpleExpression<'a>),
    String(SimpleExpression<'a>),
    Binary(ExpressionBinary<'a>),
    Unary(ExpressionUnary<'a>),
    Table(ExpressionTable<'a>),
}

#[derive(Debug, Clone)]
pub struct Assign<'a> {
    pub identifier: Token<'a>,
    pub equals: Token<'a>,
    pub value: Expression<'a>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Statement<'a> {
    Assign(Assign<'a>),
    Var(LetStatement<'a>),
    Route(Route<'a>),
}

#[derive(Debug, Clone)]
pub struct Block<'a> {
    pub body: Vec<Statement<'a>>,
    pub span: Span,
}

#[derive(Debug)]
pub struct Ast<'a> {
    pub block: Block<'a>,
}
