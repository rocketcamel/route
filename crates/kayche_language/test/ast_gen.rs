use kayche_language::ast::ast::{
    Assign, Ast, Block, Expression, LetStatement, SimpleExpression, Span, Statement, Token,
    TokenKind, VarRoot,
};

pub struct Generator {
    pub span: Span,
}

impl Generator {
    fn token(&self, value: impl Into<TokenKind>, text: String) -> Token {
        Token {
            kind: value.into(),
            text,
            span: self.span,
        }
    }

    pub fn expr_boolean(&self, value: bool) -> Expression {
        Expression::Boolean(SimpleExpression {
            token: self.token(
                if value {
                    TokenKind::True
                } else {
                    TokenKind::False
                },
                value.to_string(),
            ),
            span: self.span,
        })
    }

    pub fn stat_let(&self, identifier: &str, value: Expression) -> Statement {
        Statement::Let(LetStatement {
            root: VarRoot {
                name: self.token(TokenKind::Identifier, identifier.to_string()),
                span: self.span,
            },
            equals: self.token(TokenKind::Equals, "=".to_string()),
            value,
            span: self.span,
        })
    }

    pub fn stat_assign(&self, identifier: &str, value: Expression) -> Statement {
        Statement::Assign(Assign {
            identifier: self.token(TokenKind::Identifier, identifier.to_string()),
            equals: self.token(TokenKind::Equals, "=".to_string()),
            value,
            span: self.span,
        })
    }

    pub fn block(&self, statements: Vec<Statement>) -> Block {
        Block {
            body: statements,
            span: self.span,
        }
    }

    pub fn to_ast(block: Block) -> Ast {
        Ast { block }
    }

    pub const fn create() -> Self {
        Self {
            span: Span {
                x: 0,
                y: 0,
                z: 0,
                w: 0,
            },
        }
    }
}
