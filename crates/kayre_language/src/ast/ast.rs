use std::fmt::Display;

#[derive(Debug, Clone, Copy)]
pub struct Token {
    pub kind: TokenKind,
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

impl Display for BinaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            BinaryOperator::BinaryEquals => "compare ==",
            BinaryOperator::NEquals => "compare !=",
            BinaryOperator::Greater => "compare >",
            BinaryOperator::Less => "compare <",
            BinaryOperator::GreaterEquals => "compare >=",
            BinaryOperator::LessEquals => "compare <=",
            BinaryOperator::Add => "+",
            BinaryOperator::Subtract => "-",
            BinaryOperator::Multiply => "*",
            BinaryOperator::Divide => "/",
            BinaryOperator::Exponent => "^",
            BinaryOperator::And => "compare &&",
            BinaryOperator::Or => "compare ||",
        };

        write!(f, "{text}")
    }
}

impl Display for UnaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            UnaryOperator::Negate => "negate",
            UnaryOperator::Not => "!",
        };

        write!(f, "{text}")
    }
}

// impl Display for Token {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let kind = self.kind;

//         if kind == TokenKind::Identifier {
//             write!(f, "{}", self.text)
//         } else if kind == TokenKind::Error {
//             write!(f, "error {}", self.text)
//         } else {
//             write!(f, "{kind}")
//         }
//     }
// }

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Span {
    pub x: usize,
    pub y: usize,
    pub z: usize,
    pub w: usize,
}

#[derive(Debug, Clone)]
pub struct Delimited<VALUE> {
    pub left: Token,
    pub value: VALUE,
    pub right: Token,
}

pub type Separated<T> = Vec<Separate<T>>;

#[derive(Debug, Clone)]
pub struct Separate<T> {
    pub value: T,
    pub separator: Option<Token>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ServiceTarget {
    pub service: Token,
    pub port: usize,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct RouteTCP {
    pub target: ServiceTarget,
    pub properties: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct RouteHTTP {
    pub hostname: Token,
    pub target: ServiceTarget,
    pub properties: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Route {
    HTTP(RouteHTTP),
    TCP(RouteTCP),
}

#[derive(Debug, Clone)]
pub struct VarRoot {
    pub var: Token,
    pub name: Token,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct TableFieldNameKey {
    pub name: Token,
    pub equals: Token,
    pub value: Expression,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct TableFieldNoKey {
    pub value: Expression,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum TableField {
    NameKey(TableFieldNameKey),
    NoKey(TableFieldNoKey),
}

#[derive(Debug, Clone)]
pub struct ExpressionTable {
    pub values: Delimited<Separated<TableField>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct LetStatement {
    pub root: VarRoot,
    pub value: Expression,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct SimpleExpression {
    pub token: Token,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ExpressionBinary {
    pub left: Box<Expression>,
    pub operator: BinaryOperator,
    pub right: Box<Expression>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ExpressionUnary {
    pub operator: UnaryOperator,
    pub value: Box<Expression>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Boolean(SimpleExpression),
    Nil(SimpleExpression),
    Number(SimpleExpression),
    String(SimpleExpression),
    Binary(ExpressionBinary),
    Unary(ExpressionUnary),
    Table(ExpressionTable),
}

#[derive(Debug, Clone)]
pub struct Assign {
    pub identifier: Token,
    pub equals: Token,
    pub value: Expression,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Assign(Assign),
    Var(LetStatement),
    Route(Route),
}

#[derive(Debug, Clone)]
pub struct Block {
    pub body: Vec<Statement>,
    pub span: Span,
}

#[derive(Debug)]
pub struct Ast {
    pub block: Block,
}
