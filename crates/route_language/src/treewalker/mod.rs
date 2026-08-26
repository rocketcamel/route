pub mod types;

use std::{collections::HashMap, mem, rc::Rc};

use crate::{
    ast::ast::{
        Assign, Ast, BinaryOperator, Block, Expression, ExpressionBinary, ExpressionUnary,
        LetStatement, Route, RouteHTTP, RouteTCP, ServiceTarget, Span, Statement, TableField,
        UnaryOperator,
    },
    treewalker::types::{RawRoute, RouteKind, Value},
};

#[allow(unused)]
#[derive(Debug)]
pub struct Scope {
    up: Option<Box<Scope>>,
    root: bool,
    vars: HashMap<String, Value>,
}

#[derive(Debug)]
pub struct Issue {
    pub why: String,
    pub span: Span,
}

#[derive(Debug)]
pub struct ExecutionState {
    pub globals: HashMap<String, String>,
    pub scope: Scope,
    pub issues: Vec<Issue>,
    pub routes: Vec<RawRoute>,
}

pub struct ExecutionResult {
    pub routes: Vec<RawRoute>,
}

impl<'a> Route<'a> {
    pub fn span(&self) -> Span {
        match self {
            Route::HTTP(r) => r.span,
            Route::TCP(r) => r.span,
        }
    }
}

impl<'a> Statement<'a> {
    pub fn span(&self) -> Span {
        match self {
            Statement::Assign(n) => n.span,
            Statement::Var(n) => n.span,
            Statement::Route(n) => n.span(),
        }
    }
}

fn throw<T: Into<String>>(state: &mut ExecutionState, why: T, span: Span) {
    state.issues.push(Issue {
        why: why.into(),
        span,
    });
}

fn read_variable<'a>(state: &'a ExecutionState, var: &str) -> (Option<&'a Scope>, Option<Value>) {
    let mut active = Some(&state.scope);

    while let Some(scope) = active {
        let value = scope.vars.get(var).cloned();
        if let Some(value) = value {
            return (active, Some(value));
        }
        active = scope.up.as_deref()
    }

    return (None, None);
}

fn write_variable(state: &mut ExecutionState, var: String, new: Value) {
    let mut active = Some(&mut state.scope);

    while let Some(scope) = active {
        if scope.vars.contains_key(&var) {
            scope.vars.insert(var, new);
            return;
        }
        active = scope.up.as_deref_mut()
    }

    state.scope.vars.insert(var, new);
}

fn evaluate_binary(state: &mut ExecutionState, node: &ExpressionBinary) -> Result<Value, String> {
    let left = evaluate_expression(state, &node.left);
    let right = evaluate_expression(state, &node.right);

    match (node.operator, &left, &right) {
        (BinaryOperator::BinaryEquals, _, _) => Ok(Value::Boolean(left == right)),
        (BinaryOperator::NEquals, _, _) => Ok(Value::Boolean(left != right)),
        (BinaryOperator::Greater, Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a > b)),
        (BinaryOperator::Less, Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a < b)),
        (BinaryOperator::GreaterEquals, Value::Number(a), Value::Number(b)) => {
            Ok(Value::Boolean(a >= b))
        }
        (BinaryOperator::LessEquals, Value::Number(a), Value::Number(b)) => {
            Ok(Value::Boolean(a <= b))
        }

        (BinaryOperator::Add, Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
        (BinaryOperator::Subtract, Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)),
        (BinaryOperator::Multiply, Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),
        (BinaryOperator::Divide, Value::Number(a), Value::Number(b)) => Ok(Value::Number(a / b)),
        (BinaryOperator::Exponent, Value::Number(a), Value::Number(b)) => {
            Ok(Value::Number(a.powf(*b)))
        }

        (op, a, b) => Err(format!("attempt to {op} on {a} and {b}")),
    }
}

fn evaluate_unary(state: &mut ExecutionState, node: &ExpressionUnary) -> Result<Value, String> {
    let value = evaluate_expression(state, &node.value);

    match (node.operator, &value) {
        (UnaryOperator::Not, Value::Boolean(b)) => Ok(Value::Boolean(!b)),
        (UnaryOperator::Negate, Value::Number(a)) => Ok(Value::Number(-a)),
        (op, a) => Err(format!("attempt to {op} on {a}")),
    }
}

fn evaluate_expression(state: &mut ExecutionState, expression: &Expression) -> Value {
    match expression {
        Expression::Boolean(node) => Value::Boolean(node.token.text == "true"),
        Expression::Nil(_) => Value::Nil,
        Expression::Number(node) => Value::Number(node.token.text.parse().unwrap()),
        Expression::String(node) => Value::String(node.token.text.into()),
        Expression::Unary(node) => {
            let result = evaluate_unary(state, node);

            match result {
                Ok(r) => r,
                Err(e) => {
                    throw(state, e, node.span);
                    Value::Nil
                }
            }
        }
        Expression::Binary(node) => {
            let result = evaluate_binary(state, node);

            match result {
                Ok(r) => r,
                Err(e) => {
                    throw(state, e, node.span);
                    Value::Nil
                }
            }
        }
        Expression::Table(node) => {
            let mut table = HashMap::new();

            for field in &node.values.value {
                let token = &field.value;

                match token {
                    TableField::NoKey(_) => {
                        todo!()
                    }
                    TableField::NameKey(key) => table.insert(
                        key.name.text.to_string(),
                        evaluate_expression(state, &key.value),
                    ),
                };
            }

            Value::Table(table)
        }
    }
}

fn visit_stat_assign(state: &mut ExecutionState, assign: &Assign) {
    let key = assign.identifier.text;
    let value = evaluate_expression(state, &assign.value);

    write_variable(state, key.to_string(), value);
}

fn visit_service_target(target: &ServiceTarget) -> (Rc<str>, usize) {
    let service = target.service.text.into();
    let port = target.port;

    (service, port)
}

fn evaluate_route(state: &mut ExecutionState, block: &Block, span: Span) -> HashMap<String, Value> {
    let mut properties = HashMap::new();

    let mut inherit = |name: &str| {
        let (_, value) = read_variable(state, name);

        if let Some(value) = value {
            properties.insert(name.into(), value);
        } else {
            throw(
                state,
                format!("required property {name} not declared"),
                span,
            );
        }
    };

    inherit("gateway");
    inherit("entrypoint");

    for statement in &block.body {
        match statement {
            Statement::Assign(node) => {
                let value = evaluate_expression(state, &node.value);
                properties.insert(node.identifier.text.into(), value);
            }
            stat => {
                throw(state, "expected assignment", stat.span());
            }
        }
    }

    return properties;
}

fn visit_route_tcp(state: &mut ExecutionState, route: &RouteTCP) {
    let parent = mem::replace(
        &mut state.scope,
        Scope {
            up: None,
            root: false,
            vars: HashMap::new(),
        },
    );
    state.scope.up = Some(Box::new(parent));

    let (service_target, port) = visit_service_target(&route.target);
    let properties = evaluate_route(state, &route.properties, route.span);

    let route = RawRoute {
        kind: RouteKind::TCP,
        hostname: None,
        service_target,
        port,
        span: route.span,
        properties,
    };

    state.routes.push(route);

    if let Some(parent) = state.scope.up.take() {
        state.scope = *parent
    }
}

fn visit_route_http(state: &mut ExecutionState, route: &RouteHTTP) {
    let parent = mem::replace(
        &mut state.scope,
        Scope {
            up: None,
            root: false,
            vars: HashMap::new(),
        },
    );
    state.scope.up = Some(Box::new(parent));

    let (service_target, port) = visit_service_target(&route.target);
    let properties = evaluate_route(state, &route.properties, route.span);

    let route = RawRoute {
        kind: RouteKind::HTTP,
        hostname: Some(route.hostname.text.into()),
        service_target,
        port,
        span: route.span,
        properties,
    };

    state.routes.push(route);

    if let Some(parent) = state.scope.up.take() {
        state.scope = *parent
    }
}

fn visit_stat_route(state: &mut ExecutionState, route: &Route) {
    match route {
        Route::TCP(node) => visit_route_tcp(state, node),
        Route::HTTP(node) => visit_route_http(state, node),
    }
}

fn visit_stat_var(state: &mut ExecutionState, var: &LetStatement) {
    let name = var.root.name.text.to_string();
    let value = evaluate_expression(state, &var.value);

    write_variable(state, name, value);
}

fn visit_block(state: &mut ExecutionState, block: &Block, inherit: bool) {
    if inherit != true {
        let parent = mem::replace(
            &mut state.scope,
            Scope {
                up: None,
                root: false,
                vars: HashMap::new(),
            },
        );
        state.scope.up = Some(Box::new(parent));
    }

    for statement in &block.body {
        match statement {
            Statement::Assign(node) => visit_stat_assign(state, node),
            Statement::Route(node) => visit_stat_route(state, node),
            Statement::Var(node) => visit_stat_var(state, node),
        }
    }

    if let Some(parent) = state.scope.up.take() {
        state.scope = *parent
    }
}

pub fn create_state() -> ExecutionState {
    ExecutionState {
        globals: HashMap::new(),
        scope: Scope {
            up: None,
            root: true,
            vars: HashMap::new(),
        },
        issues: Vec::new(),
        routes: Vec::new(),
    }
}

pub fn execute(mut state: ExecutionState, ast: &Ast) -> Result<ExecutionResult, Vec<Issue>> {
    visit_block(&mut state, &ast.block, false);

    if !state.issues.is_empty() {
        return Err(state.issues);
    }

    let result_ok = ExecutionResult {
        routes: state.routes,
    };

    Ok(result_ok)
}
