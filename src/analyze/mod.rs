use crate::{
    ast::ast::Span,
    config::RouteConfig,
    treewalker::{HTTPRoute, RawRoute, RouteKind, Value},
};

pub struct Issue {
    pub why: String,
    pub span: Span,
}

pub struct Analysis {
    pub http: Vec<HTTPRoute>,
    pub issues: Vec<Issue>,
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
    let name = expect_string(state, route, "name");

    let Some(name) = name else {
        return Err(state.issues);
    };

    Ok(HTTPRoute {
        hostname: route.hostname.unwrap(),
        name,
    })
}

pub fn analyze_routes(config: &RouteConfig, routes: &[RawRoute]) {
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
