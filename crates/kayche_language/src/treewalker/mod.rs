pub mod types;

use std::{collections::HashMap, fmt::Display, mem, rc::Rc};

use crate::{
    ast::ast::{
        Assign, Ast, BinaryOperator, Block, Expression, ExpressionBinary, ExpressionUnary,
        LetStatement, Route, RouteHTTP, RouteTCP, ServiceTarget, Span, Statement, TableField,
        Token, UnaryOperator, Var,
        VarSuffix::{ExpressionIndex, NameIndex},
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

impl Route {
    pub fn span(&self) -> Span {
        match self {
            Route::HTTP(r) => r.span,
            Route::TCP(r) => r.span,
        }
    }
}

impl Statement {
    pub fn span(&self) -> Span {
        match self {
            Statement::Assign(n) => n.span,
            Statement::Let(n) => n.span,
            Statement::Route(n) => n.span(),
        }
    }
}

macro_rules! evaluate_throw {
    ($state:expr, $evaluation:expr) => {
        match $evaluation {
            Ok(value) => value,
            Err(issue) => return throw_issue($state, issue),
        }
    };
}

type Evaluation<T> = Result<T, Issue>;

fn throw<T: Into<String>>(state: &mut ExecutionState, why: T, span: Span) {
    state.issues.push(Issue {
        why: why.into(),
        span,
    });
}

fn throw_issue(state: &mut ExecutionState, issue: Issue) {
    state.issues.push(issue);
}

fn read_variable<'a>(
    state: &'a ExecutionState,
    var: &str,
) -> (Option<&'a Scope>, Option<&'a Value>) {
    let mut active = Some(&state.scope);

    while let Some(scope) = active {
        let value = scope.vars.get(var);
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

fn evaluate_binary(state: &mut ExecutionState, node: &ExpressionBinary) -> Evaluation<Value> {
    let left = evaluate_expression(state, &node.left)?;
    let right = evaluate_expression(state, &node.right)?;

    match (node.operator.kind.try_into().unwrap(), &left, &right) {
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

        (op, a, b) => Err(Issue {
            why: format!("attempt to {op} on {a} and {b}"),
            span: node.span,
        }),
    }
}

fn evaluate_unary(state: &mut ExecutionState, node: &ExpressionUnary) -> Evaluation<Value> {
    let value = evaluate_expression(state, &node.value)?;

    match (node.operator.kind.try_into().unwrap(), &value) {
        (UnaryOperator::Not, Value::Boolean(b)) => Ok(Value::Boolean(!b)),
        (UnaryOperator::Negate, Value::Number(a)) => Ok(Value::Number(-a)),
        (op, a) => Err(Issue {
            why: format!("attempt to {op} on {a}"),
            span: node.span,
        }),
    }
}

fn index<'a>(key: &Value, value: &'a Value) -> Result<&'a Value, String> {
    fn throw<Key: Display, Value: Display>(key: Key, value: Value) -> String {
        format!("could not index {key} with {value}")
    }

    match (key, value) {
        (Value::String(key), Value::Table(t)) => {
            t.get(key.as_ref()).ok_or_else(|| throw(key, "table"))
        }
        _ => Err(throw(key, value)),
    }
}

fn evaluate_var(state: &mut ExecutionState, node: &Var) -> Evaluation<Value> {
    let root = &node.root;
    let Some(mut value) = read_variable(state, &root.name.text).1.cloned() else {
        return Err(Issue {
            why: format!("could not evaluate binding {}", root.name.text),
            span: root.span,
        });
    };

    for suffix in &node.suffixes {
        match suffix {
            ExpressionIndex(suffix) => {
                let key = &suffix.node.value;

                let new = index(&evaluate_expression(state, key)?, &value).map_err(|e| Issue {
                    why: e,
                    span: node.span,
                })?;

                value = new.clone()
            }
            NameIndex(suffix) => {
                let key = &suffix.name.text;

                let new =
                    index(&Value::String(key.to_string().into()), &value).map_err(|e| Issue {
                        why: e,
                        span: node.span,
                    })?;

                value = new.clone()
            }
        }
    }

    Ok(value)
}

fn get_string_value(token: &Token) -> &str {
    &token.text[1..token.text.len() - 1]
}

fn evaluate_expression(state: &mut ExecutionState, expression: &Expression) -> Evaluation<Value> {
    match expression {
        Expression::Boolean(node) => Ok(Value::Boolean(node.token.text == "true")),
        Expression::Nil(_) => Ok(Value::Nil),
        Expression::Number(node) => Ok(Value::Number(node.token.text.parse().unwrap())),
        Expression::String(node) => Ok(Value::String(get_string_value(&node.token).into())),
        Expression::Unary(node) => evaluate_unary(state, node),

        Expression::Binary(node) => evaluate_binary(state, node),

        Expression::Table(node) => {
            let mut table = HashMap::new();

            for field in &node.values.value {
                let token = &field.value;

                match token {
                    TableField::NoKey(_) => {
                        todo!()
                    }
                    TableField::NameKey(key) => table.insert(
                        key.name.text.clone(),
                        evaluate_expression(state, &key.value)?,
                    ),
                };
            }

            Ok(Value::Table(table))
        }
        Expression::Var(node) => evaluate_var(state, node),
    }
}

fn visit_stat_assign(state: &mut ExecutionState, assign: &Assign) {
    let key = assign.identifier.text.clone();
    let value = evaluate_throw!(state, evaluate_expression(state, &assign.value));

    write_variable(state, key, value);
}

fn visit_service_target(target: &ServiceTarget) -> (Rc<str>, usize) {
    let service = target.service.text.as_str().into();
    let port = target.port;

    (service, port)
}

fn evaluate_route(state: &mut ExecutionState, block: &Block) -> Evaluation<HashMap<String, Value>> {
    let mut properties = HashMap::new();

    let mut inherit = |name: &str| -> Evaluation<()> {
        let value = read_variable(state, name).1.ok_or_else(|| Issue {
            why: format!("required property {name} not declared"),
            span: block.span,
        })?;

        properties.insert(name.into(), value.clone());
        Ok(())
    };

    inherit("gateway")?;
    inherit("entrypoint")?;

    for statement in &block.body {
        match statement {
            Statement::Assign(node) => {
                let value = evaluate_expression(state, &node.value)?;
                properties.insert(node.identifier.text.clone(), value);
            }
            stat => {
                throw(state, "expected assignment", stat.span());
            }
        }
    }

    Ok(properties)
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
    let properties = evaluate_throw!(state, evaluate_route(state, &route.properties));

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
    let properties = evaluate_throw!(state, evaluate_route(state, &route.properties));

    let route = RawRoute {
        kind: RouteKind::HTTP,
        hostname: Some(get_string_value(&route.hostname).into()),
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

fn visit_stat_let(state: &mut ExecutionState, binding: &LetStatement) {
    let name = binding.root.name.text.clone();
    let value = evaluate_throw!(state, evaluate_expression(state, &binding.value));

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
            Statement::Let(node) => visit_stat_let(state, node),
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
