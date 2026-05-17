use std::{collections::HashMap, mem};

use crate::ast::ast::{
    Assign, Ast, Block, Expression, LetStatement, Route, RouteKind, ServiceTarget, Span, Statement,
};

#[derive(Debug)]
pub struct Scope {
    up: Option<Box<Scope>>,
    root: bool,
    vars: HashMap<String, Value>,
}

#[derive(Debug)]
pub struct Issue {
    why: String,
    span: Span,
}

#[derive(Debug)]
pub enum RouteResult {
    HTTP(HTTPRoute),
    TCP(TCPRoute),
}

#[derive(Debug)]
pub struct ExecutionState {
    pub globals: HashMap<String, String>,
    pub scope: Scope,
    pub issues: Vec<Issue>,
    pub routes: Vec<RouteResult>,
}

pub struct ExecutionResult {
    pub http: Vec<HTTPRoute>,
    pub tcp: Vec<TCPRoute>,
}

#[derive(Clone, PartialEq, Debug)]
enum Value {
    String(String),
    Number(i64),
    Bool(bool),
    Nil,
}

impl Value {
    fn to_string(&self) -> Option<String> {
        match self {
            Value::String(n) => Some(n.to_string()),
            Value::Number(n) => Some(n.to_string()),
            Value::Bool(n) => Some(n.to_string()),
            Value::Nil => None,
        }
    }
}

impl<'a> Statement<'a> {
    pub fn span(&self) -> Span {
        match self {
            Statement::Assign(n) => n.span,
            Statement::Var(n) => n.span,
            Statement::Route(n) => n.span,
        }
    }
}

#[derive(Debug)]
pub struct HTTPRoute {
    pub name: String,
    pub hostname: String,
    pub service: String,
    pub port: usize,
    pub namespace: String,
    pub gateway: String,
    pub entrypoint: String,
}

#[derive(Debug)]
pub struct TCPRoute {
    pub name: String,
    pub service: String,
    pub port: usize,
    pub namespace: String,
    pub gateway: String,
    pub entrypoint: String,
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
        Expression::String(node) => Value::String(node.token.text.to_string()),
    }
}

fn visit_stat_assign(state: &mut ExecutionState, assign: &Assign) {
    let key = assign.identifier.text;
    let value = evaluate_expression(&assign.value);

    write_variable(state, key.to_string(), value);
}

fn visit_service_target(target: &ServiceTarget) -> (String, usize) {
    let service = target.service.text.to_string();
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

fn visit_stat_route(state: &mut ExecutionState, route: &Route) {
    evaluate_route(state, &route.properties);

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
                None => {
                    throw(state, format!("missing required property {name}"), span);
                    result_ok = false;
                }
            }
        }

        result_ok.then_some(props)
    }
    let Some(props) = expect_props(state, &["gateway", "namespace", "entrypoint"], route.span)
    else {
        return;
    };

    let name;
    if let Some(n) = read_variable(state, "name").1 {
        let value = n.to_string();
        if value.is_none() {
            throw(state, "no name for route definition", route.span);
            return;
        }
        name = value.unwrap()
    } else {
        let Some(n) = route.hostname.text.split(".").next() else {
            return;
        };
        name = n.to_string()
    }

    let (service, port) = visit_service_target(&route.target);

    match route.kind {
        RouteKind::HTTP => {
            let route = HTTPRoute {
                name,
                gateway: props["gateway"].to_string().unwrap(),
                namespace: props["namespace"].to_string().unwrap(),
                entrypoint: props["entrypoint"].to_string().unwrap(),
                hostname: route.hostname.text.to_string(),
                service,
                port,
            };
            state.routes.push(RouteResult::HTTP(route));
        }
        RouteKind::TCP => {
            let route = TCPRoute {
                name,
                gateway: props["gateway"].to_string().unwrap(),
                namespace: props["namespace"].to_string().unwrap(),
                entrypoint: props["entrypoint"].to_string().unwrap(),
                service,
                port,
            };
            state.routes.push(RouteResult::TCP(route));
        }
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

    let mut result_ok = ExecutionResult {
        http: Vec::new(),
        tcp: Vec::new(),
    };

    for route in state.routes {
        match route {
            RouteResult::HTTP(node) => result_ok.http.push(node),
            RouteResult::TCP(node) => result_ok.tcp.push(node),
        }
    }

    Ok(result_ok)
}
