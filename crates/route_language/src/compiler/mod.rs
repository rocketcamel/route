use crate::ast::ast::{
    Assign, Ast, Block, Expression, ExpressionTable, LetStatement, Route, RouteHTTP, RouteTCP,
    Statement, TableField, TokenKind, VarRoot,
};

#[derive(Debug, Clone)]
pub struct VMState {
    pub instructions: Vec<Instruction>,
    pub locals: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum RouteKind {
    HTTP,
    TCP,
}

#[derive(Debug, Clone, Copy)]
pub struct InstructionPushTable {
    pub alloc: usize,
}

#[derive(Debug, Clone)]
pub struct InstructionPushRoute {
    pub r#type: RouteKind,
    pub service_target: String,
    pub service_port: usize,
    pub hostname: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Instruction {
    PushNumber(f64),
    PushString(String),
    PushBoolean(bool),
    PushNil,
    PushLocal(usize),
    PushGlobal(String),
    PushTable(InstructionPushTable),
    PushRoute(InstructionPushRoute),

    SetLocal(usize),
    SetGlobal(String),
    SetTable,
    SetRouteProperty(String),
    SetRoute,
}

pub struct Compiler {
    pub vm_state: VMState,
    pub next_instrucion: usize,
}

impl Compiler {
    fn INSERT(&mut self, instruction: Instruction) {
        self.vm_state.instructions.push(instruction);
        self.next_instrucion += 1
    }

    fn DEFINE_LOCAL(&mut self, root: VarRoot) -> usize {
        self.vm_state.locals.push(root.name.text.to_string());
        return self.vm_state.locals.len() - 1;
    }

    fn GET_VALUE(&mut self, key: &str) {
        let index = self.vm_state.locals.iter().position(|v| v == key);

        if let Some(index) = index {
            self.INSERT(Instruction::PushLocal(index));
        } else {
            self.INSERT(Instruction::PushGlobal(key.to_string()));
        }
    }

    fn SET_VALUE(&mut self, key: &str) {
        let index = self.vm_state.locals.iter().position(|v| v == key);

        if let Some(index) = index {
            self.INSERT(Instruction::SetLocal(index));
        } else {
            self.INSERT(Instruction::SetGlobal(key.to_string()));
        }
    }

    fn compile_expression(&mut self, expression: Expression) {
        match expression {
            Expression::Boolean(node) => {
                self.INSERT(Instruction::PushBoolean(
                    if node.token.kind == TokenKind::True {
                        true
                    } else {
                        false
                    },
                ));
            }
            Expression::Nil(_) => {
                self.INSERT(Instruction::PushNil);
            }
            Expression::Number(node) => {
                self.INSERT(Instruction::PushNumber(node.token.text.parse().unwrap()));
            }
            Expression::String(node) => {
                self.INSERT(Instruction::PushString(node.token.text.to_string()));
            }
            Expression::Table(node) => {
                self.compile_table(node);
            }
        }
    }

    fn compile_table(&mut self, table: ExpressionTable) {
        self.INSERT(Instruction::PushTable(InstructionPushTable { alloc: 1 }));

        for value in table.values.value {
            match value.value {
                TableField::NameKey(node) => {
                    self.INSERT(Instruction::PushString(node.name.text.to_string()));
                    self.compile_expression(node.value);
                    self.INSERT(Instruction::SetTable);
                }
                TableField::NoKey(node) => {
                    self.compile_expression(node.value);
                }
            }
        }
    }

    fn compile_assignment(&mut self, assign: Assign) {
        self.compile_expression(assign.value);
        self.SET_VALUE(assign.identifier.text);
    }

    fn compile_var(&mut self, node: LetStatement) {
        self.compile_expression(node.value);
        let slot = self.DEFINE_LOCAL(node.root);
        self.INSERT(Instruction::SetLocal(slot));
    }

    fn compile_route_block(&mut self, block: Block) {
        for statement in block.body {
            match statement {
                Statement::Assign(node) => {
                    self.compile_expression(node.value);
                    self.INSERT(Instruction::SetRouteProperty(
                        node.identifier.text.to_string(),
                    ));
                }
                _ => {}
            }
        }
    }

    fn compile_route_tcp(&mut self, tcp: RouteTCP) {
        self.INSERT(Instruction::PushRoute(InstructionPushRoute {
            r#type: RouteKind::TCP,
            service_target: tcp.target.service.text.to_string(),
            service_port: tcp.target.port,
            hostname: None,
        }));

        self.compile_route_block(tcp.properties);
        self.INSERT(Instruction::SetRoute);
    }

    fn compile_route_http(&mut self, http: RouteHTTP) {
        self.INSERT(Instruction::PushRoute(InstructionPushRoute {
            r#type: RouteKind::HTTP,
            service_target: http.target.service.text.to_string(),
            service_port: http.target.port,
            hostname: Some(http.hostname.text.to_string()),
        }));

        self.compile_route_block(http.properties);
        self.INSERT(Instruction::SetRoute);
    }

    fn compile_route(&mut self, route: Route) {
        match route {
            Route::TCP(node) => self.compile_route_tcp(node),
            Route::HTTP(node) => self.compile_route_http(node),
        }
    }

    fn compile_block(&mut self, block: Block) {
        for statement in block.body {
            match statement {
                Statement::Assign(node) => self.compile_assignment(node),
                Statement::Var(node) => self.compile_var(node),
                Statement::Route(node) => self.compile_route(node),
            }
        }
    }

    pub fn compile(&mut self, ast: Ast) -> Vec<Instruction> {
        self.compile_block(ast.block);

        return self.vm_state.instructions.clone();
    }
}
