use crate::ast::ast::{Assign, Ast, Block, Expression, LetStatement, Route, Statement};

#[allow(unused)]
pub trait Visitor<'a> {
    fn visit_ast(&mut self, ast: &Ast<'a>) {
        self.visit_block(&ast.block);
    }

    fn visit_block(&mut self, block: &Block<'a>) {
        for statement in &block.body {
            self.visit_statement(statement);
        }
    }

    fn visit_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::Assign(node) => self.visit_assign(node),
            Statement::Route(node) => self.visit_route(node),
            Statement::Var(node) => self.visit_var(node),
        }
    }

    fn visit_expression(&mut self, expression: &Expression) {}
    fn visit_assign(&mut self, assign: &Assign) {}
    fn visit_route(&mut self, route: &Route) {}
    fn visit_var(&mut self, var: &LetStatement) {}
}
