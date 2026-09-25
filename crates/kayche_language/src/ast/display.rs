use crate::ast::ast::{
    Assign, Block, Delimited, Expression, ExpressionTable, LetStatement, Route, RouteHTTP,
    RouteTCP, Separated, ServiceTarget, Span, Statement, TableField, Token, TokenKind, Var,
    VarRoot, VarSuffix,
};

#[allow(unused)]
pub struct Display {
    pub source: Vec<u8>,
    pub line_length: usize,
    pub pos: usize,
    pub tbs: usize,
}

#[allow(unused)]
impl Display {
    fn write_char(&mut self, char: u8) {
        self.source.push(char);
        self.line_length += 1;
        self.pos += 1
    }

    fn write_str(&mut self, str: &str) {
        self.source.append(&mut str.as_bytes().to_vec());
        self.line_length += str.len();
        self.pos += str.len();
    }

    fn write_line(&mut self) {
        self.write_char(b'\n');
        self.write_str(&"\t".repeat(self.tbs));
    }

    fn display_token(&mut self, token: &Token) {
        self.write_str(&token.text);

        match token.kind {
            TokenKind::Let => self.write_char(b' '),
            _ => {}
        }
    }

    fn display_separated<T, F: FnMut(&mut Self, &T)>(
        &mut self,
        separated: &Separated<T>,
        mut call: F,
    ) {
        for value in separated {
            call(self, &value.value);
            if let Some(separator) = &value.separator {
                self.display_token(separator);
            }
        }
    }

    fn display_tablefield(&mut self, field: &TableField) {
        match field {
            TableField::NameKey(field) => {
                self.display_token(&field.name);
                self.display_token(&field.equals);
                self.display_expression(&field.value);
            }
            TableField::NoKey(field) => self.display_expression(&field.value),
        }
    }

    fn display_delimited<T, F: FnMut(&mut Self, &T)>(
        &mut self,
        delimited: &Delimited<T>,
        mut call: F,
    ) {
        self.display_token(&delimited.left);
        call(self, &delimited.value);
        self.display_token(&delimited.right);
    }

    fn display_table(&mut self, table: &ExpressionTable) {
        self.display_delimited(&table.values, |display, separated| {
            display.display_separated(separated, |display, field| {
                display.display_tablefield(&field)
            })
        });
    }

    fn display_expression(&mut self, expression: &Expression) {
        match expression {
            Expression::Binary(expr) => {
                self.display_expression(&expr.left);
                self.display_token(&expr.operator);
                self.display_expression(&expr.right);
            }
            Expression::Unary(expr) => {
                self.display_token(&expr.operator);
                self.display_expression(&expr.value);
            }
            Expression::Table(expr) => {
                self.display_table(&expr);
            }
            Expression::String(expr) => self.display_token(&expr.token),
            Expression::Number(expr) => self.display_token(&expr.token),
            Expression::Boolean(expr) => self.display_token(&expr.token),
            Expression::Nil(expr) => self.display_token(&expr.token),
            Expression::Var(expr) => self.display_var(expr),
        }
    }

    fn display_assign(&mut self, stat: &Assign) {
        self.display_token(&stat.identifier);
        self.display_token(&stat.equals);
        self.display_expression(&stat.value);
    }

    fn display_var_suffix(&mut self, suffix: &VarSuffix) {
        match suffix {
            VarSuffix::ExpressionIndex(suffix) => {
                self.display_token(&suffix.period);
                self.display_delimited(&suffix.node, |display, node| {
                    display.display_expression(node)
                });
            }
            VarSuffix::NameIndex(suffix) => {
                self.display_token(&suffix.period);
                self.display_token(&suffix.name);
            }
        }
    }

    fn display_var_root(&mut self, root: &VarRoot) {
        self.display_token(&root.name);
    }

    fn display_var(&mut self, var: &Var) {
        self.display_var_root(&var.root);

        for suffix in &var.suffixes {
            self.display_var_suffix(suffix);
        }

        self.write_char(b' ');
    }

    fn display_service_target(&mut self, target: &ServiceTarget) {
        self.display_token(&target.service);
        self.display_token(&target.equals);
        self.display_token(&Token {
            kind: TokenKind::Number,
            text: target.port.to_string(),
            span: Span {
                x: 0,
                y: 0,
                z: 0,
                w: 0,
            },
        });
    }

    fn display_route_http(&mut self, route: &RouteHTTP) {
        self.display_token(&route.hostname);
        self.display_service_target(&route.target);

        self.write_char(b'{');
        self.tbs += 1;

        self.display_block(&route.properties);
        self.tbs -= 1;
        self.write_char(b'}');
    }

    fn display_route_tcp(&mut self, route: &RouteTCP) {
        self.display_service_target(&route.target);
        self.write_char(b'{');
        self.tbs += 1;

        self.display_block(&route.properties);
        self.tbs -= 1;
        self.write_char(b'}');
    }

    fn display_route(&mut self, route: &Route) {
        self.write_str("route ");

        match route {
            Route::HTTP(route) => self.display_route_http(route),
            Route::TCP(route) => self.display_route_tcp(route),
        }
    }

    fn display_stat_let(&mut self, stat: &LetStatement) {
        self.write_str("let ");
        self.display_var_root(&stat.root);
        self.display_token(&stat.equals);
        self.display_expression(&stat.value);
    }

    pub fn display_block(&mut self, block: &Block) -> String {
        self.write_line();

        for statement in &block.body {
            match statement {
                Statement::Assign(stat) => self.display_assign(stat),
                Statement::Let(stat) => self.display_stat_let(stat),
                Statement::Route(stat) => self.display_route(stat),
            }

            self.write_line();
        }

        String::from_utf8_lossy(&self.source).to_string()
    }

    pub fn create() -> Self {
        Self {
            pos: 0,
            line_length: 0,
            source: Vec::new(),
            tbs: 0,
        }
    }
}
