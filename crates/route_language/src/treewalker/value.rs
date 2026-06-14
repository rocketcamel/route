use std::{collections::HashMap, fmt::Display, rc::Rc};

#[derive(Clone, PartialEq, Debug)]
pub enum Value {
    String(Rc<str>),
    Number(i64),
    Bool(bool),
    Nil,
    Table(HashMap<String, Value>),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(_) => write!(f, "string"),
            Value::Number(_) => write!(f, "number"),
            Value::Bool(_) => write!(f, "bool"),
            Value::Nil => write!(f, "nil"),
            Value::Table(_) => write!(f, "table"),
        }
    }
}

impl Value {
    pub fn to_string(&self) -> Option<Rc<str>> {
        match self {
            Value::String(n) => Some(n.clone()),
            Value::Number(n) => Some(n.to_string().into()),
            Value::Bool(n) => Some(n.to_string().into()),
            Value::Nil => None,
            Value::Table(_) => None,
        }
    }

    pub fn truthy(&self) -> bool {
        match self {
            Value::Bool(n) => *n == true,
            Value::Nil => false,
            _ => true,
        }
    }
}
