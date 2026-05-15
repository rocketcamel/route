use std::collections::HashMap;

use crate::ast::{
    Visitor,
    ast::{Ast, Expression, Route, RouteKind, Span, Statement, Token},
};

pub struct Scope<'a, 'scope> {
    up: Option<&'scope Scope<'a, 'scope>>,
    root: bool,
    vars: HashMap<&'a str, &'a str>,
}

pub struct ExecutionState {
    pub globals: HashMap<String, String>,
}

#[derive(Debug)]
pub struct Issue<'a> {
    why: &'a str,
    span: Span,
}

#[derive(Clone, Copy, PartialEq)]
enum Value<'a> {
    Str(&'a str),
    Number(i64),
    Bool(bool),
    Nil,
}

impl<'a> Value<'a> {
    fn to_string(&self) -> Option<String> {
        match self {
            Value::Str(n) => Some(n.to_string()),
            Value::Number(n) => Some(n.to_string()),
            Value::Bool(n) => Some(n.to_string()),
            Value::Nil => None,
        }
    }
}

#[derive(Debug)]
pub struct HTTPRoute {
    pub name: String,
    pub hostname: String,
    pub service: String,
    pub namespace: String,
    pub gateway: String,
}

#[derive(Debug)]
pub struct TCPRoute {
    pub name: String,
    pub service: String,
    pub namespace: String,
    pub gateway: String,
}

pub struct Renderer<'a> {
    pub output: String,
    pub state: ExecutionState,
    pub issues: Vec<Issue<'a>>,
}

impl<'a> Renderer<'a> {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            state: ExecutionState {
                globals: HashMap::new(),
            },
            issues: Vec::new(),
        }
    }

    pub fn render(&mut self, ast: &Ast<'a>) -> (&[Issue<'a>], &str) {
        self.visit_ast(ast);
        (&self.issues, &self.output)
    }

    fn read_var<'b, 'scope>(
        &self,
        scope: &'b Scope,
        var: &str,
    ) -> (Option<&'b Scope<'b, 'scope>>, Option<String>) {
        let mut active = Some(scope);

        while let Some(scope) = active {
            let value = scope.vars.get(var).copied();
            if let Some(value) = value {
                return (Some(scope), Some(value.to_string()));
            }
            active = scope.up
        }
        let value = self.state.globals.get(var).cloned().map(|v| v.to_string());

        (None, value)
    }

    fn evaluate_expression(&self, expression: &Expression<'a>) -> Value<'a> {
        match expression {
            Expression::Boolean(n) => Value::Bool(n.token.text == "true"),
            Expression::Number(n) => Value::Number(n.token.text.parse().unwrap()),
            Expression::String(n) => Value::Str(&n.token.text),
            Expression::Nil(_) => Value::Nil,
        }
    }

    fn emit_http_route(&mut self, route: &HTTPRoute) {
        println!("{route:?}");
    }

    fn emit_tcp_route(&mut self, route: &HTTPRoute) {
        println!("{route:?}");
    }
}

impl<'a> Visitor<'a> for Renderer<'a> {
    fn visit_route(&mut self, route: &crate::ast::ast::Route) {
        let scope = Scope {
            up: None,
            root: true,
            vars: HashMap::new(),
        };
        let mut properties = HashMap::new();

        for statement in &route.properties.body {
            if let Statement::Assign(node) = statement {
                let value = self.evaluate_expression(&node.value);
                if value == Value::Nil {
                    self.issues.push(Issue {
                        why: "nil properties not supported",
                        span: node.span,
                    });
                    return;
                }

                properties.insert(node.identifier.text, value);
            }
        }

        match route.kind {
            RouteKind::HTTP => {
                let Some(namespace) = properties
                    .get("namespace")
                    .copied()
                    .map(|n| n.to_string().unwrap())
                else {
                    return;
                };
                let (_, Some(gateway)) = self.read_var(&scope, "gateway") else {
                    self.issues.push(Issue {
                        why: "gateway not specified",
                        span: route.span,
                    });
                    return;
                };

                let route = HTTPRoute {
                    name: route.hostname.text.split(".").next().unwrap().to_string(),
                    hostname: route.hostname.text.to_string(),
                    namespace,
                    service: route.target.service.text.to_string(),
                    gateway,
                };
                self.emit_http_route(&route);
            }
            _ => {}
        }
    }

    fn visit_var(&mut self, var: &crate::ast::ast::LetStatement) {
        let value = self.evaluate_expression(&var.value).to_string().unwrap();

        self.state
            .globals
            .insert(var.var.name.text.to_string(), value);
    }

    fn visit_assign(&mut self, _: &crate::ast::ast::Assign) {}
    fn visit_expression(&mut self, _: &Expression) {}
}
