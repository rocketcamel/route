mod value;

use std::{collections::HashMap, mem, rc::Rc};

pub use value::Value;

use crate::ast::ast::{
    Assign, Ast, Block, Expression, LetStatement, Route, RouteHTTP, RouteTCP, ServiceTarget, Span,
    Statement, TableField,
};

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

#[derive(Debug, PartialEq)]
pub enum RouteKind {
    HTTP,
    TCP,
}

#[derive(Debug)]
pub struct RawRoute {
    pub kind: RouteKind,
    pub hostname: Option<Rc<str>>,
    pub service: Rc<str>,
    pub port: usize,
    pub span: Span,

    pub properties: HashMap<String, Value>,
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

fn evaluate_expression(expression: &Expression) -> Value {
    match expression {
        Expression::Boolean(node) => Value::Bool(node.token.text == "true"),
        Expression::Nil(_) => Value::Nil,
        Expression::Number(node) => Value::Number(node.token.text.parse().unwrap()),
        Expression::String(node) => Value::String(node.token.text.into()),
        Expression::Table(node) => {
            let mut table = HashMap::new();

            for field in &node.values.value {
                let token = &field.value;

                match token {
                    TableField::NoKey(_) => {
                        todo!()
                    }
                    TableField::NameKey(key) => {
                        table.insert(key.name.text.to_string(), evaluate_expression(&key.value))
                    }
                };
            }

            Value::Table(table)
        }
    }
}

fn visit_stat_assign(state: &mut ExecutionState, assign: &Assign) {
    let key = assign.identifier.text;
    let value = evaluate_expression(&assign.value);

    write_variable(state, key.to_string(), value);
}

fn visit_service_target(target: &ServiceTarget) -> (Rc<str>, usize) {
    let service = target.service.text.into();
    let port = target.port;

    (service, port)
}

fn evaluate_route(state: &mut ExecutionState, block: &Block) {
    for statement in &block.body {
        match statement {
            Statement::Assign(node) => visit_stat_assign(state, node),
            stat => {
                throw(state, "expected assignment", stat.span());
            }
        }
    }
}

fn expect_props(
    state: &mut ExecutionState,
    names: &[&str],
    span: Span,
) -> Option<HashMap<String, Value>> {
    let mut props = HashMap::new();
    let mut result_ok = true;

    for &name in names {
        match read_variable(state, name).1 {
            Some(value) => {
                props.insert(name.to_string(), value);
            }
            _ => {
                throw(state, format!("missing required property {name}"), span);
                result_ok = false;
            }
        }
    }

    result_ok.then_some(props)
}

fn route_private(state: &mut ExecutionState) -> bool {
    let Some(property) = read_variable(state, "private").1 else {
        return true;
    };

    property.truthy()
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

    evaluate_route(state, &route.properties);

    let (service, port) = visit_service_target(&route.target);
    let mut properties = HashMap::new();
    let mut scope_ref = Some(&state.scope);

    while let Some(scope) = scope_ref {
        for (k, v) in &scope.vars {
            properties.entry(k.clone()).or_insert_with(|| v.clone());
        }

        scope_ref = scope.up.as_deref()
    }

    state.routes.push(RawRoute {
        kind: RouteKind::TCP,
        hostname: None,
        service,
        port,
        span: route.span,
        properties,
    });

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

    evaluate_route(state, &route.properties);

    let (service, port) = visit_service_target(&route.target);
    let mut properties = HashMap::new();
    let mut scope_ref = Some(&state.scope);

    while let Some(scope) = scope_ref {
        for (k, v) in &scope.vars {
            properties.entry(k.clone()).or_insert_with(|| v.clone());
        }

        scope_ref = scope.up.as_deref()
    }

    state.routes.push(RawRoute {
        kind: RouteKind::HTTP,
        hostname: Some(route.hostname.text.into()),
        service,
        port,
        span: route.span,
        properties,
    });

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
    let value = evaluate_expression(&var.value);

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
