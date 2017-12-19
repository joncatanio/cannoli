extern crate cannoli;

use cannoli::lexer::Lexer;
use cannoli::parser;
use cannoli::parser::ast::*;

#[test]
fn test_keyword_global() {
    let stream = Lexer::new("global var1, var2, var3\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::Global {
                names: vec![String::from("var1"), String::from("var2"),
                    String::from("var3")]
            }
        ]
    };
    assert_eq!(ast, expected);
}

#[test]
fn test_keyword_nonlocal() {
    let stream = Lexer::new("nonlocal var1, var2, var3\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::Nonlocal {
                names: vec![String::from("var1"), String::from("var2"),
                    String::from("var3")]
            }
        ]
    };
    assert_eq!(ast, expected);
}

#[test]
fn test_pass() {
    let stream = Lexer::new("pass;pass;pass;pass\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::Pass, Statement::Pass, Statement::Pass, Statement::Pass
        ]
    };
    assert_eq!(ast, expected);

    let stream = Lexer::new("pass;pass;pass;pass;\n");
    let ast = parser::parse_start_symbol(stream);
    assert_eq!(ast, expected);
}

#[test]
fn test_empty_return() {
    let stream = Lexer::new("return\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![Statement::Return { value: None }]
    };
    assert_eq!(ast, expected);
}
