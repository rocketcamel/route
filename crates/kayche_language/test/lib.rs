use kayche_language::ast::{
    Parser,
    ast::{
        Ast,
        BinaryOperator::{
            self, Add, And, BinaryEquals, Divide, Exponent, Greater, GreaterEquals, Less,
            LessEquals, Multiply, NEquals, Or, Subtract,
        },
        Block, TokenKind,
    },
    display::Display,
};

use crate::ast_gen::Generator;

mod ast_gen;

fn test_ast(input: &Ast) {
    let mut display = Display::create();
    let s = display.display_block(&input.block);
    println!("{s}");

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

const BINARY_OPERATORS: [BinaryOperator; 13] = [
    // operators
    BinaryEquals,
    NEquals,
    Greater,
    Less,
    GreaterEquals,
    LessEquals,
    // arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    Exponent,
    // ternary
    And,
    Or,
];

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

    #[test]
    fn nil() {
        case_ast(GEN.block(vec![GEN.stat_let("test", GEN.expr_nil())]));
    }

    #[test]
    fn number() {
        case_ast(GEN.block(vec![GEN.stat_let("test", GEN.expr_number(1234.0))]));
        case_ast(GEN.block(vec![GEN.stat_let(
            "test2",
            GEN.expr_unary(GEN.token(TokenKind::Negate, "-"), GEN.expr_number(1234.0)),
        )]));
    }

    #[test]
    fn string() {
        case_ast(GEN.block(vec![GEN.stat_let("test", GEN.expr_string("test1234"))]));
        case_ast(GEN.block(vec![
            GEN.stat_let("test", GEN.expr_string(&"test1234".repeat(1000))),
        ]));
    }

    #[test]
    fn table() {
        case_ast(GEN.block(vec![GEN.stat_let(
            "test",
            GEN.expr_table(vec![
                GEN.tablefield_namekey(
                    "test123",
                    GEN.expr_table(vec![
                        GEN.tablefield_namekey("test31231", GEN.expr_boolean(true)),
                    ]),
                ),
                GEN.tablefield_namekey("gdaskjl", GEN.expr_string("done")),
            ]),
        )]));
    }

    #[test]
    fn binary_operators() {
        for operator in BINARY_OPERATORS {
            case_ast(GEN.block(vec![GEN.stat_let(
                "test",
                GEN.expr_evaluate(GEN.expr_binary(
                    GEN.expr_number(10.0),
                    GEN.token(operator, operator.to_string()),
                    GEN.expr_number(20.0),
                )),
            )]));
        }
    }

    #[test]
    fn var() {
        case_ast(GEN.block(vec![
            GEN.stat_let(
                "test",
                GEN.expr_table(vec![
                    GEN.tablefield_namekey("test123", GEN.expr_boolean(true)),
                    GEN.tablefield_namekey("testanother", GEN.expr_boolean(false)),
                ]),
            ),
            GEN.stat_assign(
                "test2",
                GEN.expr_var(
                    GEN.var_root("test"),
                    vec![GEN.varsuffix_name_index("test123")],
                ),
            ),
            GEN.stat_assign(
                "test3",
                GEN.expr_var(
                    GEN.var_root("test"),
                    vec![GEN.varsuffix_name_index("testanother")],
                ),
            ),
        ]));
    }
}
