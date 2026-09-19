use kayche_language::ast::ast::{Assign, Expression, Span, Statement, Token, TokenKind};

pub struct Generator {
    pub span: Span,
}

impl Generator {
    fn token<T: Into<TokenKind>>(&self, value: T) -> Token {
        Token {
            kind: value.into(),
            span: self.span,
        }
    }

    fn stat_assign(&self, identifier: &str, value: Expression) -> Statement {
        Statement::Assign(Assign {
            identifier: self.token(identifier),
        })
    }
}
