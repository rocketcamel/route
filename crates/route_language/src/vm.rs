use std::{collections::HashMap, fmt::Display, rc::Rc};

use crate::{
    ast::ast::Span,
    compiler::{Instruction, RouteKind},
};

#[derive(Debug, Clone)]
pub enum Value {
    Boolean(bool),
    Nil,
    Number(f64),
    String(Rc<str>),
    Table(HashMap<String, Value>),
    Route(RawRoute),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Value::Boolean(_) => "bool",
            Value::Nil => "nil",
            Value::Number(_) => "number",
            Value::String(_) => "string",
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

#[derive(Debug, Clone)]
pub struct RawRoute {
    pub kind: RouteKind,
    pub hostname: Option<Rc<str>>,
    pub service_target: Rc<str>,
    pub port: usize,
    pub properties: HashMap<String, Value>,
    pub span: Span,
}

pub struct VirtualMachine {
    pub locals: Vec<Value>,
    pub globals: HashMap<String, Value>,
    pub instruction_at: usize,
    pub instruction_end: usize,
    pub stack: Vec<Value>,
    pub n: usize,
    pub routes: Vec<RawRoute>,
}

impl VirtualMachine {
    fn PUSH(&mut self, value: Value) {
        println!("pushing value to stack, {:?}", value);
        self.stack.push(value);
        self.n += 1;
    }

    fn POP(&mut self) -> Value {
        println!(
            "popping {} from stack, popped {:?}",
            self.n,
            self.stack[self.n - 1].clone()
        );
        self.n -= 1;
        return self.stack.pop().unwrap();
    }

    fn GET(&mut self, at: usize) -> Option<&mut Value> {
        if at > 0 {
            return None;
        }
        return self.stack.get_mut(at);
    }

    fn process(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::PushNumber(n) => self.PUSH(Value::Number(n)),
            Instruction::PushString(s) => self.PUSH(Value::String(s.into())),
            Instruction::PushBoolean(b) => self.PUSH(Value::Boolean(b)),
            Instruction::PushNil => self.PUSH(Value::Nil),
            Instruction::PushLocal(index) => self.PUSH(self.locals[index].clone()),
            Instruction::PushGlobal(key) => self.PUSH(self.globals[&key].clone()),
            Instruction::PushTable(table) => {
                self.PUSH(Value::Table(HashMap::with_capacity(table.alloc)))
            }
            Instruction::PushRoute(route) => self.PUSH(Value::Route(RawRoute {
                kind: route.r#type,
                hostname: route.hostname.map(|s| s.into()),
                service_target: route.service_target.into(),
                port: route.service_port,
                properties: HashMap::new(),
                span: route.span,
            })),

            Instruction::SetLocal(index) => {
                let value = self.POP();

                if index >= self.locals.len() {
                    self.locals.resize(index + 1, Value::Nil);
                }
                self.locals[index] = value
            }
            Instruction::SetGlobal(key) => {
                let value = self.POP();
                self.globals.insert(key, value);
            }
            Instruction::SetTable => {
                let (value, Value::String(key)) = (self.POP(), self.POP()) else {
                    unreachable!();
                };
                let Value::Table(t) = self.GET(self.n - 1).unwrap() else {
                    unreachable!()
                };

                t.insert(key.to_string(), value);
            }
            Instruction::SetRouteProperty(key) => {
                let value = self.POP();
                let Value::Route(route) = self.GET(self.n - 1).unwrap() else {
                    unreachable!()
                };
                route.properties.insert(key, value);
            }
            Instruction::SetRoute => {
                let Value::Route(route) = self.POP() else {
                    unreachable!()
                };
                self.routes.push(route);
            }
        }
    }

    pub fn create_vm() -> Self {
        Self {
            locals: Vec::new(),
            globals: HashMap::new(),
            instruction_at: 0,
            instruction_end: 0,
            n: 0,
            stack: Vec::new(),
            routes: Vec::new(),
        }
    }

    pub fn run(&mut self, instructions: Vec<Instruction>) -> Vec<RawRoute> {
        self.instruction_at = 0;
        self.instruction_end = instructions.len() - 1;

        while self.instruction_at <= self.instruction_end {
            self.process(instructions[self.instruction_at].clone());
            self.instruction_at += 1
        }

        return self.routes.clone();
    }
}
