extern crate cannoli;

use cannoli::lexer::Lexer;
use cannoli::parser;
use cannoli::parser::ast::*;

#[test]
fn keyword_global() {
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
fn keyword_nonlocal() {
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
fn pass() {
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
fn empty_return() {
    let stream = Lexer::new("return\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![Statement::Return { value: None }]
    };
    assert_eq!(ast, expected);
}

#[test]
fn or_and_test_expr() {
    let stream =
        Lexer::new("return True or False and False or True and False\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::Return { value: Some(Expression::BoolOp {
                op: BoolOperator::Or,
                values: vec![
                    Expression::NameConstant { value: Singleton::True },
                    Expression::BoolOp {
                        op: BoolOperator::And,
                        values: vec![
                            Expression::NameConstant { value: Singleton::False },
                            Expression::NameConstant { value: Singleton::False },
                        ]
                    },
                    Expression::BoolOp {
                        op: BoolOperator::And,
                        values: vec![
                            Expression::NameConstant { value: Singleton::True},
                            Expression::NameConstant { value: Singleton::False },
                        ]
                    }
                ]
            }) }
        ]
    };
    assert_eq!(ast, expected);
}

#[test]
fn comparison() {
    let stream = Lexer::new("return True < False > True <= False >= \
                            True != True in False not in True is False \
                            is not True\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::Return { value: Some(Expression::Compare {
                left: Box::new(
                    Expression::NameConstant { value: Singleton::True }
                ),
                ops: vec![
                    CmpOperator::LT,
                    CmpOperator::GT,
                    CmpOperator::LE,
                    CmpOperator::GE,
                    CmpOperator::NE,
                    CmpOperator::In,
                    CmpOperator::NotIn,
                    CmpOperator::Is,
                    CmpOperator::IsNot
                ],
                comparators: vec![
                    Expression::NameConstant { value: Singleton::False },
                    Expression::NameConstant { value: Singleton::True },
                    Expression::NameConstant { value: Singleton::False },
                    Expression::NameConstant { value: Singleton::True },
                    Expression::NameConstant { value: Singleton::True },
                    Expression::NameConstant { value: Singleton::False },
                    Expression::NameConstant { value: Singleton::True },
                    Expression::NameConstant { value: Singleton::False },
                    Expression::NameConstant { value: Singleton::True },
                ]
            }) }
        ]
    };
    assert_eq!(ast, expected);
}

#[test]
fn return_call_expr() {
    let stream = Lexer::new("return func(1, \"test\", True, *d, **e,)\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::Return { value: Some(Expression::Call {
                func: Box::new(Expression::Name {
                    id: String::from("func"),
                    ctx: ExprContext::Load
                }),
                args: vec![
                    Expression::Num {n: Number::DecInteger(String::from("1"))},
                    Expression::Str {s: String::from("test")},
                    Expression::NameConstant {value: Singleton::True},
                    Expression::Starred {
                        value: Box::new(Expression::Name {
                            id: String::from("d"),
                            ctx: ExprContext::Load
                        }),
                        ctx: ExprContext::Load
                    }
                ],
                keywords: vec![
                    Keyword::Keyword {
                        arg: None,
                        value: Expression::Name {
                            id: String::from("e"),
                            ctx: ExprContext::Load
                        }
                    }
                ]
            })}
        ]
    };
    assert_eq!(ast, expected);
}

#[test]
fn return_nested_call() {
    let stream = Lexer::new("return f()()()\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::Return { value: Some(Expression::Call {
                func: Box::new(Expression::Call {
                    func: Box::new(Expression::Call {
                        func: Box::new(Expression::Name {
                            id: String::from("f"),
                            ctx: ExprContext::Load
                        }),
                        args: vec![],
                        keywords: vec![]
                    }),
                    args: vec![],
                    keywords: vec![]
                }),
                args: vec![],
                keywords: vec![]
            })}
        ]
    };
    assert_eq!(ast, expected);
}
