use std::collections::HashMap;

#[derive(Clone, PartialEq, Debug)]
pub enum Value {
    String(String),
    Number(i64),
    Bool(bool),
    Nil,
    Table(HashMap<String, Value>),
}

impl Value {
    pub fn to_string(&self) -> Option<String> {
        match self {
            Value::String(n) => Some(n.to_string()),
            Value::Number(n) => Some(n.to_string()),
            Value::Bool(n) => Some(n.to_string()),
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
