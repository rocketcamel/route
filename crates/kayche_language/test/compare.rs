use std::{fmt::Debug, mem::discriminant};

use kayche_language::ast::ast::{
    Assign, Ast, Block, Delimited, Expression, LetStatement, Route, RouteHTTP, RouteTCP, Separate,
    Separated, ServiceTarget, Statement, TableField, Token, VarRoot, VarSuffix,
};

#[derive(Debug, Clone)]
pub struct Issue<'a> {
    pub why: &'static str,
    pub a: Option<&'a dyn Debug>,
    pub b: Option<&'a dyn Debug>,
}

pub struct Compare<'a> {
    pub issues: Vec<Issue<'a>>,
}

impl<'a> Compare<'a> {
    pub fn throw(&mut self, why: &'static str, a: Option<&'a dyn Debug>, b: Option<&'a dyn Debug>) {
        self.issues.push(Issue { why, a, b });
    }

    pub fn compare_token(&mut self, a: &'a Token, b: &'a Token) {
        if a.kind != b.kind {
            self.throw("mismatching kinds", Some(a), Some(b));
        }
        if a.text != b.text {
            self.throw("mismatching text", Some(a), Some(b));
        }
    }

    pub fn compare_delimited<V, F: FnOnce(&mut Self, &'a V, &'a V) -> ()>(
        &mut self,
        a: &'a Delimited<V>,
        b: &'a Delimited<V>,
        call: F,
    ) {
        call(self, &a.value, &b.value);
    }

    pub fn compare_separated<V: Debug, F: FnMut(&mut Self, &'a Separate<V>, &'a Separate<V>)>(
        &mut self,
        a: &'a Separated<V>,
        b: &'a Separated<V>,
        mut call: F,
    ) {
        let total = a.len().max(b.len());

        for i in 0..total {
            let a = a.get(i);
            let b = b.get(i);

            match (a, b) {
                (Some(a), Some(b)) => {
                    if a.separator.is_none() || b.separator.is_none() {
                        return self.throw("missing separator", Some(a), Some(b));
                    }

                    call(self, a, b)
                }
                (a, b) => {
                    self.throw(
                        "missing separated",
                        a.map(|n| n as &dyn Debug),
                        b.map(|n| n as &dyn Debug),
                    );
                }
            }
        }
    }

    pub fn compare_tablefield(&mut self, a: &'a TableField, b: &'a TableField) {
        if discriminant(a) != discriminant(b) {
            return self.throw("mismatching tables", Some(a), Some(b));
        }

        match (a, b) {
            (TableField::NameKey(a), TableField::NameKey(b)) => {
                self.compare_token(&a.name, &b.name);
                self.compare_expression(&a.value, &b.value);
            }
            (TableField::NoKey(a), TableField::NoKey(b)) => {
                self.compare_expression(&a.value, &b.value);
            }
            _ => unreachable!("unsupported tablefield {a:?} {b:?}"),
        }
    }

    pub fn compare_var_root(&mut self, a: &'a VarRoot, b: &'a VarRoot) {
        self.compare_token(&a.name, &b.name);
    }

    pub fn compare_var_suffix(&mut self, a: &'a VarSuffix, b: &'a VarSuffix) {
        if discriminant(a) != discriminant(b) {
            return self.throw("mismatching varsuffixes", Some(a), Some(b));
        }

        match (a, b) {
            (VarSuffix::ExpressionIndex(a), VarSuffix::ExpressionIndex(b)) => {
                self.compare_delimited(&a.node, &b.node, |compare, a, b| {
                    compare.compare_expression(a, b)
                });
            }
            (VarSuffix::NameIndex(a), VarSuffix::NameIndex(b)) => {
                self.compare_token(&a.name, &b.name);
            }
            _ => unreachable!("unsupported varsuffix"),
        }
    }

    pub fn compare_expression(&mut self, a: &'a Expression, b: &'a Expression) {
        if discriminant(a) != discriminant(b) {
            return self.throw("mismatching expressions", Some(a), Some(b));
        }

        match (a, b) {
            (Expression::Boolean(a), Expression::Boolean(b))
            | (Expression::Nil(a), Expression::Nil(b))
            | (Expression::Number(a), Expression::Number(b))
            | (Expression::String(a), Expression::String(b)) => {
                self.compare_token(&a.token, &b.token);
            }
            (Expression::Binary(a), Expression::Binary(b)) => {
                self.compare_expression(&a.left, &b.left);
                self.compare_token(&a.operator, &b.operator);
                self.compare_expression(&a.right, &b.right);
            }
            (Expression::Unary(a), Expression::Unary(b)) => {
                self.compare_token(&a.operator, &b.operator);
                self.compare_expression(&a.value, &b.value);
            }
            (Expression::Table(a), Expression::Table(b)) => {
                self.compare_delimited(&a.values, &b.values, |compare, a, b| {
                    compare.compare_separated(a, b, |compare, a, b| {
                        compare.compare_tablefield(&a.value, &b.value)
                    })
                });
            }
            (Expression::Var(a), Expression::Var(b)) => {
                self.compare_var_root(&a.root, &b.root);

                for (a, b) in a.suffixes.iter().zip(&b.suffixes) {
                    self.compare_var_suffix(a, b);
                }
            }
            (Expression::Evaluate(a), Expression::Evaluate(b)) => {
                self.compare_delimited(&a.value, &b.value, |compare, a, b| {
                    compare.compare_expression(a, b)
                });
            }
            _ => unreachable!("unsupported expression"),
        }
    }

    pub fn compare_assign(&mut self, a: &'a Assign, b: &'a Assign) {
        self.compare_token(&a.identifier, &b.identifier);
    }

    pub fn compare_stat_let(&mut self, a: &'a LetStatement, b: &'a LetStatement) {
        self.compare_var_root(&a.root, &b.root);
        self.compare_expression(&a.value, &b.value);
    }

    pub fn compare_service_target(&mut self, a: &'a ServiceTarget, b: &'a ServiceTarget) {
        self.compare_token(&a.service, &b.service);

        if a.port != b.port {
            return self.throw("mismatched service target ports", Some(a), Some(b));
        }
    }

    pub fn compare_route_http(&mut self, a: &'a RouteHTTP, b: &'a RouteHTTP) {
        self.compare_token(&a.hostname, &b.hostname);
        self.compare_service_target(&a.target, &b.target);
        self.compare_block(&a.properties, &b.properties);
    }

    pub fn compare_route_tcp(&mut self, a: &'a RouteTCP, b: &'a RouteTCP) {
        self.compare_service_target(&a.target, &b.target);
        self.compare_block(&a.properties, &b.properties);
    }

    pub fn compare_route(&mut self, a: &'a Route, b: &'a Route) {
        if discriminant(a) != discriminant(b) {
            return self.throw("mismatched routes", Some(a), Some(b));
        }

        match (a, b) {
            (Route::HTTP(a), Route::HTTP(b)) => self.compare_route_http(a, b),
            (Route::TCP(a), Route::TCP(b)) => self.compare_route_tcp(a, b),
            _ => unreachable!("unsupported route"),
        }
    }

    pub fn compare_statement(&mut self, a: &'a Statement, b: &'a Statement) {
        if discriminant(a) != discriminant(b) {
            return self.throw("mismatching statements", Some(a), Some(b));
        }

        match (a, b) {
            (Statement::Assign(a), Statement::Assign(b)) => self.compare_assign(a, b),
            (Statement::Let(a), Statement::Let(b)) => self.compare_stat_let(a, b),
            (Statement::Route(a), Statement::Route(b)) => self.compare_route(a, b),
            _ => unreachable!("unsupported statement"),
        }
    }

    pub fn compare_block(&mut self, a: &'a Block, b: &'a Block) {
        let total = a.body.len().max(b.body.len());

        for i in 0..total {
            let stat_a = a.body.get(i);
            let stat_b = b.body.get(i);

            match (stat_a, stat_b) {
                (Some(a), Some(b)) => {
                    self.compare_statement(a, b);
                }
                (a, b) => {
                    self.throw(
                        "missing statement",
                        a.map(|n| n as &dyn Debug),
                        b.map(|n| n as &dyn Debug),
                    );
                }
            }
        }
    }

    pub fn compare_ast(&mut self, a: &'a Ast, b: &'a Ast) -> Vec<Issue<'a>> {
        self.issues.clear();
        self.compare_block(&a.block, &b.block);
        self.issues.clone()
    }

    pub const fn create() -> Self {
        Self { issues: Vec::new() }
    }
}
