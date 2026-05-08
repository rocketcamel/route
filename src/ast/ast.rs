#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
}

#[derive(Debug)]
pub struct Gateway {
    pub name: Identifier,
    pub span: Span,
}

#[derive(Debug)]
pub struct ServiceTarget {
    pub service: Identifier,
    pub port: usize,
    pub span: Span,
}

#[derive(Debug)]
pub enum PropertyKind {
    Namespace,
}

#[derive(Debug)]
pub struct Property {
    pub kind: PropertyKind,
    pub value: Identifier,
    pub span: Span,
}

#[derive(Debug)]
pub struct Route {
    pub hostname: Identifier,
    pub target: ServiceTarget,
    pub properties: Block,
    pub span: Span,
}

#[derive(Debug)]
pub struct Identifier {
    pub value: String,
    pub span: Span,
}

#[derive(Debug)]
pub enum Statement {
    Gateway(Gateway),
    Route(Route),
    Property(Property),
}

#[derive(Debug)]
pub struct Block {
    pub body: Vec<Statement>,
    pub span: Span,
}

#[derive(Debug)]
pub struct Ast {
    pub block: Block,
}
