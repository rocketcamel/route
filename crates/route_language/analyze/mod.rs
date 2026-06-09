use crate::{
    ast::ast::Span,
    treewalker::{RawRoute, RouteKind, Value},
};

pub struct Issue {
    pub why: String,
    pub span: Span,
}

pub struct Analysis {
    pub http: Vec<HTTPRoute>,
    pub issues: Vec<Issue>,
}

#[derive(Debug)]
pub struct HTTPRoute {
    pub name: String,
    pub hostname: String,
    pub service: String,
    pub port: usize,
    pub namespace: String,
    pub gateway: Gateway,
    pub private: bool,
}

#[derive(Debug)]
pub struct TCPRoute {
    pub name: String,
    pub service: String,
    pub port: usize,
    pub namespace: String,
    pub gateway: Gateway,
    pub entrypoint: String,
    pub private: bool,
}

#[derive(Debug)]
pub struct Gateway {
    pub name: String,
    pub namespace: String,
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

fn expect_string(state: &mut Analysis, route: &RawRoute, name: &str) -> Option<String> {
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

fn analyze_http(state: &mut Analysis, route: &RawRoute) -> Result<HTTPRoute, Vec<Issue>> {
    let name = expect_string(state, route, "name").ok_or(state.issues)?;
    let namespace = expect_string(state, route, "namespace").ok_or(state.issues)?;
    let gateway = expect_string(state, route, "namespace").ok_or(state.issues)?;

    Ok(HTTPRoute {
        hostname: route.hostname.unwrap(),
        name,
        namespace,
        gateway: Gateway { name, namespace },
    })
}

pub fn analyze_routes(routes: &[RawRoute]) {
    let state = Analysis {
        http: Vec::new(),
        issues: Vec::new(),
    };

    for route in routes {
        match route.kind {
            RouteKind::HTTP => analyze_http(&mut state, route),
            RouteKind::TCP => analyze_tcp(route),
        }
    }
}
