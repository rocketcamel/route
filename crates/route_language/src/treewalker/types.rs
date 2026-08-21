use std::{collections::HashMap, fmt::Display, rc::Rc};

use crate::ast::ast::Span;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RouteKind {
    HTTP,
    TCP,
}

#[derive(Debug, Clone)]
pub struct RawRoute {
    pub kind: RouteKind,
    pub hostname: Option<Rc<str>>,
    pub service_target: Rc<str>,
    pub port: usize,
    pub properties: HashMap<String, Value>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum Value {
    String(Rc<str>),
    Number(f64),
    Boolean(bool),
    Nil,
    Table(HashMap<String, Value>),
    Route(RawRoute),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Value::String(_) => "string",
            Value::Number(_) => "number",
            Value::Boolean(_) => "boolean",
            Value::Nil => "nil",
            Value::Table(_) => "table",
            Value::Route(_) => "route",
        };

        write!(f, "{text}")
    }
}

impl Value {
    pub fn to_string(&self) -> Option<Rc<str>> {
        match self {
            Value::Boolean(n) => Some(n.to_string().into()),
            Value::String(n) => Some(n.clone()),
            Value::Number(n) => Some(n.to_string().into()),
            _ => None,
        }
    }
}
