use crate::ast::ast::{
    Assign, Block, Delimited, Expression, ExpressionTable, LetStatement, Route, RouteHTTP,
    RouteTCP, Separated, ServiceTarget, Span, Statement, TableField, Token, TokenKind,
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

    pub fn display_token(&mut self, token: &Token) {
        self.write_str(&token.text);

        match token.kind {
            TokenKind::Let => self.write_char(b' '),
            _ => {}
        }
    }

    pub fn display_separated<T, F: FnMut(&mut Self, &T)>(
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

    pub fn display_tablefield(&mut self, field: &TableField) {
        match field {
            TableField::NameKey(field) => {
                self.display_token(&field.name);
                self.display_token(&field.equals);
                self.display_expression(&field.value);
            }
            TableField::NoKey(field) => self.display_expression(&field.value),
        }
    }

    pub fn display_delimited<T, F: FnMut(&mut Self, &T)>(
        &mut self,
        delimited: &Delimited<T>,
        mut call: F,
    ) {
        self.display_token(&delimited.left);
        call(self, &delimited.value);
        self.display_token(&delimited.right);
    }

    pub fn display_table(&mut self, table: &ExpressionTable) {
        self.display_delimited(&table.values, |display, separated| {
            display.display_separated(separated, |display, field| {
                display.display_tablefield(&field)
            })
        });
    }

    pub fn display_expression(&mut self, expression: &Expression) {
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
        }
    }

    pub fn display_assign(&mut self, stat: &Assign) {
        self.display_token(&stat.identifier);
        self.display_token(&stat.equals);
        self.display_expression(&stat.value);
    }

    pub fn display_var(&mut self, var: &LetStatement) {
        self.display_token(&var.root.var);
        self.display_token(&var.root.name);
        self.display_expression(&var.value);
    }

    pub fn display_service_target(&mut self, target: &ServiceTarget) {
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

    pub fn display_route_http(&mut self, route: &RouteHTTP) {
        self.display_token(&route.hostname);
        self.display_service_target(&route.target);
        self.display_block(&route.properties);
    }

    pub fn display_route_tcp(&mut self, route: &RouteTCP) {
        self.display_service_target(&route.target);
        self.display_block(&route.properties);
    }

    pub fn display_route(&mut self, route: &Route) {
        match route {
            Route::HTTP(route) => self.display_route_http(route),
            Route::TCP(route) => self.display_route_tcp(route),
        }
    }

    pub fn display_block(&mut self, block: &Block) {
        for statement in &block.body {
            match statement {
                Statement::Assign(stat) => self.display_assign(stat),
                Statement::Var(stat) => self.display_var(stat),
                Statement::Route(stat) => self.display_route(stat),
            }
        }
    }
}
