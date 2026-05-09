use crate::ast::Token;

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
}

#[derive(Debug)]
pub struct Gateway<'a> {
    pub token: Token<'a>,
    pub span: Span,
}

#[derive(Debug)]
pub struct ServiceTarget<'a> {
    pub service: Token<'a>,
    pub port: usize,
    pub span: Span,
}

#[derive(Debug)]
pub enum PropertyKind {
    Namespace,
}

#[derive(Debug)]
pub struct Property<'a> {
    pub kind: PropertyKind,
    pub token: Token<'a>,
    pub span: Span,
}

#[derive(Debug)]
pub struct Route<'a> {
    pub hostname: Token<'a>,
    pub target: ServiceTarget<'a>,
    pub properties: Block<'a>,
    pub span: Span,
}

#[derive(Debug)]
pub enum Statement<'a> {
    Gateway(Gateway<'a>),
    Route(Route<'a>),
    Property(Property<'a>),
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
