use kayche_language::ast::ast::{
    Assign, Ast, Block, Delimited, Expression, ExpressionBinary, ExpressionEvaluate,
    ExpressionTable, ExpressionUnary, LetStatement, Separate, Separated, SimpleExpression, Span,
    Statement, TableField, TableFieldNameKey, Token, TokenKind, Var, VarRoot,
    VarSuffix::{self, ExpressionIndex},
    VarSuffixExpressionIndex, VarSuffixNameIndex,
};

pub struct Generator {
    pub span: Span,
}

impl Generator {
    pub fn token(&self, value: impl Into<TokenKind>, text: impl Into<String>) -> Token {
        Token {
            kind: value.into(),
            text: text.into(),
            span: self.span,
        }
    }

    pub fn delimit<V>(&self, left: Token, right: Token, value: V) -> Delimited<V> {
        Delimited {
            left: left,
            value,
            right: right,
        }
    }

    pub fn separate<T>(&self, values: Vec<T>) -> Separated<T> {
        let mut map = Vec::new();

        for (i, value) in values.into_iter().enumerate() {
            map.insert(
                i,
                Separate {
                    value,
                    separator: Some(self.token(TokenKind::Comma, ",")),
                    span: self.span,
                },
            );
        }

        map
    }

    pub fn tablefield_namekey(&self, name: &str, value: Expression) -> TableField {
        TableField::NameKey(TableFieldNameKey {
            name: self.token(TokenKind::Identifier, name),
            equals: self.token(TokenKind::Equals, "="),
            value,
            span: self.span,
        })
    }

    pub fn expr_table(&self, fields: Vec<TableField>) -> Expression {
        Expression::Table(ExpressionTable {
            values: self.delimit(
                self.token(TokenKind::LBrace, "{"),
                self.token(TokenKind::RBrace, "}"),
                self.separate(fields),
            ),
            span: self.span,
        })
    }

    pub fn var_root(&self, name: &str) -> VarRoot {
        VarRoot {
            name: self.token(TokenKind::Identifier, name),
            span: self.span,
        }
    }

    pub fn varsuffix_name_index(&self, name: &str) -> VarSuffix {
        VarSuffix::NameIndex(VarSuffixNameIndex {
            period: self.token(TokenKind::Period, "."),
            name: self.token(TokenKind::Identifier, name),
            span: self.span,
        })
    }

    pub fn varsuffix_expression_index(&self, value: Expression) -> VarSuffix {
        VarSuffix::ExpressionIndex(VarSuffixExpressionIndex {
            period: self.token(TokenKind::Period, "."),
            node: self.delimit(
                self.token(TokenKind::LBracketSquare, "["),
                self.token(TokenKind::RBracketSquare, "]"),
                value,
            ),
            span: self.span,
        })
    }

    pub fn expr_var(&self, root: VarRoot, suffixes: Vec<VarSuffix>) -> Expression {
        Expression::Var(Var {
            root,
            suffixes,
            span: self.span,
        })
    }

    pub fn expr_evaluate(&self, value: Expression) -> Expression {
        Expression::Evaluate(ExpressionEvaluate {
            value: self.delimit(
                self.token(TokenKind::LBracket, "("),
                self.token(TokenKind::RBracket, ")"),
                Box::new(value),
            ),
            span: self.span,
        })
    }

    pub fn expr_unary(&self, op: Token, value: Expression) -> Expression {
        Expression::Unary(ExpressionUnary {
            operator: op,
            value: value.into(),
            span: self.span,
        })
    }

    pub fn expr_binary(&self, left: Expression, op: Token, right: Expression) -> Expression {
        Expression::Binary(ExpressionBinary {
            left: left.into(),
            operator: op,
            right: right.into(),
            span: self.span,
        })
    }

    pub fn expr_string(&self, text: &str) -> Expression {
        Expression::String(SimpleExpression {
            token: self.token(TokenKind::String, format!("\"{text}\"")),
            span: self.span,
        })
    }

    pub fn expr_number(&self, number: f64) -> Expression {
        Expression::Number(SimpleExpression {
            token: self.token(TokenKind::Number, number.to_string()),
            span: self.span,
        })
    }

    pub fn expr_nil(&self) -> Expression {
        Expression::Nil(SimpleExpression {
            token: self.token(TokenKind::Nil, "nil"),
            span: self.span,
        })
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
                name: self.token(TokenKind::Identifier, identifier),
                span: self.span,
            },
            equals: self.token(TokenKind::Equals, "="),
            value,
            span: self.span,
        })
    }

    pub fn stat_assign(&self, identifier: &str, value: Expression) -> Statement {
        Statement::Assign(Assign {
            identifier: self.token(TokenKind::Identifier, identifier),
            equals: self.token(TokenKind::Equals, "="),
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
