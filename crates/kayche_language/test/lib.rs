use kayche_language::ast::{
    Parser,
    ast::{Ast, Block},
    display::Display,
};

use crate::ast_gen::Generator;

mod ast_gen;

fn test_ast(input: &Ast) {
    let mut display = Display::create();
    let s = display.display_block(&input.block);

    let mut parser = Parser::new(s.as_bytes()).unwrap();
    let result = parser.parse();

    let ast = match result {
        Ok(ast) => ast,
        Err(error) => panic!("{error:?}"),
    };
}

fn case_ast(block: Block) {
    let ast = Generator::to_ast(block);
    test_ast(&ast);
}

const GEN: Generator = Generator::create();

#[cfg(test)]
mod basic_expression_parsing {
    use super::*;

    #[test]
    fn boolean() {
        case_ast(GEN.block(vec![
            GEN.stat_let("test", GEN.expr_boolean(true)),
            GEN.stat_let("test2", GEN.expr_boolean(false)),
        ]));
    }
}
