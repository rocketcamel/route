use std::{collections::HashMap, rc::Rc};

use crate::{
    ast::ast::Span,
    treewalker::types::{RawRoute, RouteKind, Value},
};

#[derive(Debug)]
pub struct Issue {
    pub why: String,
    pub span: Span,
}

pub struct Analysis {
    pub http: Vec<HTTPRoute>,
    pub tcp: Vec<TCPRoute>,
    pub issues: Vec<Issue>,
}

#[derive(Debug)]
pub struct HTTPRoute {
    pub name: Rc<str>,
    pub hostname: Rc<str>,
    pub service: Rc<str>,
    pub port: usize,
    pub namespace: Rc<str>,
    pub gateway: Gateway,
    pub private: bool,
}

#[derive(Debug)]
pub struct TCPRoute {
    pub name: Rc<str>,
    pub service: Rc<str>,
    pub port: usize,
    pub namespace: Rc<str>,
    pub gateway: Gateway,
    pub entrypoint: Rc<str>,
    pub private: bool,
}

#[derive(Debug)]
pub struct Gateway {
    pub name: Rc<str>,
    pub namespace: Rc<str>,
}

fn expect<'a>(state: &mut Analysis, route: &'a RawRoute, name: &str) -> Option<&'a Value> {
    let Some(property) = route.properties.get(name) else {
        state.issues.push(Issue {
            why: format!("missing required property '{name}'"),
            span: route.span,
        });

        return None;
    };

    Some(property)
}

fn expect_or<'a>(route: &'a RawRoute, name: &str, default: &'a Value) -> &'a Value {
    let Some(property) = route.properties.get(name) else {
        return default;
    };

    property
}

fn expect_boolean<'a>(route: &'a RawRoute, name: &str) -> Option<bool> {
    let value = expect_or(route, name, &Value::Boolean(false));

    if let Value::Boolean(bool) = value {
        Some(*bool)
    } else {
        unreachable!()
    }
}

fn expect_string(state: &mut Analysis, route: &RawRoute, name: &str) -> Option<Rc<str>> {
    let value = expect(state, route, name)?;

    match value.to_string() {
        Some(s) => Some(s),
        None => {
            state.issues.push(Issue {
                why: format!("'{name}' must be a string"),
                span: route.span,
            });

            return None;
        }
    }
}

#[allow(unused)]
fn expect_number(state: &mut Analysis, route: &RawRoute, name: &str) -> Option<f64> {
    let value = expect(state, route, name)?;

    match value {
        Value::Number(n) => Some(*n),
        r#type => {
            state.issues.push(Issue {
                why: format!(
                    "invalid type '{type}' for '{name}' expected number, but got '{value}'"
                ),
                span: route.span,
            });

            return None;
        }
    }
}

fn expect_table<'a>(
    state: &mut Analysis,
    route: &'a RawRoute,
    name: &str,
) -> Option<&'a HashMap<String, Value>> {
    let value = expect(state, route, name)?;

    match value {
        Value::Table(node) => Some(node),
        _ => {
            state.issues.push(Issue {
                why: format!("'{name}' must be a table"),
                span: route.span,
            });

            return None;
        }
    }
}

fn analyze_gateway(state: &mut Analysis, route: &RawRoute) -> Option<Gateway> {
    let table = expect_table(state, route, "gateway")?;

    let mut validate_property = |name: &str| {
        let Some(value) = table.get(name) else {
            state.issues.push(Issue {
                why: format!("gateway declaration missing required field '{name}'"),
                span: route.span,
            });

            return None;
        };

        match value.to_string() {
            Some(data) => Some(data),
            None => {
                state.issues.push(Issue {
                    why: format!(
                        "invalid type for field '{name}', expected string, but got '{value}'",
                    ),
                    span: route.span,
                });

                return None;
            }
        }
    };

    let name = validate_property("name")?;
    let namespace = validate_property("namespace")?;

    Some(Gateway { name, namespace })
}

fn analyze_name(state: &mut Analysis, route: &RawRoute, hostname: &str) -> Option<Rc<str>> {
    let value = expect_string(state, route, "name");

    match value {
        Some(name) => Some(name),
        None => {
            let name: Option<Rc<str>> = hostname.split(".").next().map(|n| n.into());

            if name.is_none() {
                state.issues.push(Issue {
                    why: format!("missing required property 'name', unable to parse hostname to grab it automatically"),
                    span: route.span
                });
            }

            return name;
        }
    }
}

fn analyze_http(state: &mut Analysis, route: &RawRoute) -> Option<HTTPRoute> {
    let hostname = match route.hostname.as_ref() {
        Some(h) => h.clone(),
        None => {
            state.issues.push(Issue {
                why: format!("http route missing required hostname"),
                span: route.span,
            });

            return None;
        }
    };

    let name = analyze_name(state, route, &hostname)?;
    let namespace = expect_string(state, route, "namespace")?;
    let private = expect_boolean(route, "private")?;
    let gateway = analyze_gateway(state, route)?;

    Some(HTTPRoute {
        hostname,
        name,
        namespace,
        gateway,
        port: route.port,
        service: route.service_target.clone(),
        private,
    })
}

fn analyze_tcp(state: &mut Analysis, route: &RawRoute) -> Option<TCPRoute> {
    let name = expect_string(state, route, "name")?;
    let namespace = expect_string(state, route, "namespace")?;
    let entrypoint = expect_string(state, route, "entrypoint")?;
    let private = expect_boolean(route, "private")?;
    let gateway = analyze_gateway(state, route)?;

    Some(TCPRoute {
        name,
        namespace,
        entrypoint,
        gateway,
        port: route.port,
        service: route.service_target.clone(),
        private,
    })
}

pub fn analyze_routes(routes: &[RawRoute]) -> Analysis {
    let mut state = Analysis {
        http: Vec::new(),
        tcp: Vec::new(),
        issues: Vec::new(),
    };

    for route in routes {
        if route.kind == RouteKind::TCP {
            let result = analyze_tcp(&mut state, route);

            if let Some(result) = result {
                state.tcp.push(result);
            }
        } else if route.kind == RouteKind::HTTP {
            let result = analyze_http(&mut state, route);

            if let Some(result) = result {
                state.http.push(result);
            }
        }
    }

    return state;
}
