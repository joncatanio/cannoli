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
                            Expression::NameConstant {
                                value: Singleton::False },
                            Expression::NameConstant {
                                value: Singleton::False },
                        ]
                    },
                    Expression::BoolOp {
                        op: BoolOperator::And,
                        values: vec![
                            Expression::NameConstant {
                                value: Singleton::True},
                            Expression::NameConstant {
                                value: Singleton::False },
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

#[test]
fn slices_and_indexes_1() {
    let stream = Lexer::new("return p[0]\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::Return { value: Some(Expression::Subscript {
                value: Box::new(Expression::Name {
                    id: String::from("p"),
                    ctx: ExprContext::Load
                }),
                slice: Box::new(Slice::Index {
                    value: Expression::Num {
                        n: Number::DecInteger(String::from("0"))
                    }
                }),
                ctx: ExprContext::Load
            })}
        ]
    };
    assert_eq!(ast, expected);
}

#[test]
fn slices_and_indexes_2() {
    let stream = Lexer::new("return p[0,]\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::Return { value: Some(Expression::Subscript {
                value: Box::new(Expression::Name {
                    id: String::from("p"),
                    ctx: ExprContext::Load
                }),
                slice: Box::new(Slice::Index {
                    value: Expression::Tuple {
                        elts: vec![
                            Expression::Num {
                                n: Number::DecInteger(String::from("0"))
                            }
                        ],
                        ctx: ExprContext::Load
                    }
                }),
                ctx: ExprContext::Load
            })}
        ]
    };
    assert_eq!(ast, expected);
}

#[test]
fn slices_and_indexes_3() {
    let stream = Lexer::new("return p[0,a]\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::Return { value: Some(Expression::Subscript {
                value: Box::new(Expression::Name {
                    id: String::from("p"),
                    ctx: ExprContext::Load
                }),
                slice: Box::new(Slice::Index {
                    value: Expression::Tuple {
                        elts: vec![
                            Expression::Num {
                                n: Number::DecInteger(String::from("0"))
                            },
                            Expression::Name {
                                id: String::from("a"),
                                ctx: ExprContext::Load
                            }
                        ],
                        ctx: ExprContext::Load
                    }
                }),
                ctx: ExprContext::Load
            })}
        ]
    };
    assert_eq!(ast, expected);

    // Add trailing comma, should result in the same AST
    let stream = Lexer::new("return p[0,a,]\n");
    let ast = parser::parse_start_symbol(stream);
    assert_eq!(ast, expected);
}

#[test]
fn slices_and_indexes_4() {
    let stream = Lexer::new("return p[1:4:-1]\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::Return { value: Some(Expression::Subscript {
                value: Box::new(Expression::Name {
                    id: String::from("p"),
                    ctx: ExprContext::Load
                }),
                slice: Box::new(Slice::Slice {
                    lower: Some(Expression::Num {
                        n: Number::DecInteger(String::from("1"))
                    }),
                    upper: Some(Expression::Num {
                        n: Number::DecInteger(String::from("4"))
                    }),
                    step: Some(Expression::UnaryOp {
                        op: UnaryOperator::USub,
                        operand: Box::new(Expression::Num {
                            n: Number::DecInteger(String::from("1"))
                        })
                    })
                }),
                ctx: ExprContext::Load
            })}
        ]
    };
    assert_eq!(ast, expected);
}

#[test]
fn slices_and_indexes_5() {
    let stream = Lexer::new("return p[1:4:-1,]\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::Return { value: Some(Expression::Subscript {
                value: Box::new(Expression::Name {
                    id: String::from("p"),
                    ctx: ExprContext::Load
                }),
                slice: Box::new(Slice::ExtSlice {
                    dims: vec![
                        Slice::Slice {
                            lower: Some(Expression::Num {
                                n: Number::DecInteger(String::from("1"))
                            }),
                            upper: Some(Expression::Num {
                                n: Number::DecInteger(String::from("4"))
                            }),
                            step: Some(Expression::UnaryOp {
                                op: UnaryOperator::USub,
                                operand: Box::new(Expression::Num {
                                    n: Number::DecInteger(String::from("1"))
                                })
                            })
                        }
                    ]
                }),
                ctx: ExprContext::Load
            })}
        ]
    };
    assert_eq!(ast, expected);
}

#[test]
fn slices_and_indexes_6() {
    let stream = Lexer::new("return p[:]\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::Return { value: Some(Expression::Subscript {
                value: Box::new(Expression::Name {
                    id: String::from("p"),
                    ctx: ExprContext::Load
                }),
                slice: Box::new(Slice::Slice {
                    lower: None,
                    upper: None,
                    step: None
                }),
                ctx: ExprContext::Load
            })}
        ]
    };
    assert_eq!(ast, expected);
}

#[test]
fn slices_and_indexes_7() {
    let stream = Lexer::new("return p[:,0]\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::Return { value: Some(Expression::Subscript {
                value: Box::new(Expression::Name {
                    id: String::from("p"),
                    ctx: ExprContext::Load
                }),
                slice: Box::new(Slice::ExtSlice {
                    dims: vec![
                        Slice::Slice {
                            lower: None,
                            upper: None,
                            step: None
                        },
                        Slice::Index {
                            value: Expression::Num {
                                n: Number::DecInteger(String::from("0"))
                            }
                        }
                    ]
                }),
                ctx: ExprContext::Load
            })}
        ]
    };
    assert_eq!(ast, expected);
}

#[test]
fn yield_no_arg() {
    let stream = Lexer::new("yield\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::Expr { value: Expression::Yield { value: None } }
        ]
    };
    assert_eq!(ast, expected);
}

#[test]
fn yield_testlist_single() {
    let stream = Lexer::new("yield 1\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::Expr { value: Expression::Yield {
                value: Some(Box::new(Expression::Num {
                    n: Number::DecInteger(String::from("1"))
                }))
            }}
        ]
    };
    assert_eq!(ast, expected);
}

#[test]
fn yield_testlist_tuple() {
    let stream = Lexer::new("yield 1,\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::Expr { value: Expression::Yield {
                value: Some(Box::new(Expression::Tuple {
                    elts: vec![Expression::Num {
                        n: Number::DecInteger(String::from("1"))
                    }],
                    ctx: ExprContext::Load
                }))
            }}
        ]
    };
    assert_eq!(ast, expected);
}

#[test]
fn yield_from_simple() {
    let stream = Lexer::new("yield from 1\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::Expr { value: Expression::YieldFrom {
                value: Box::new(Expression::Num {
                    n: Number::DecInteger(String::from("1"))
                })
            }}
        ]
    };
    assert_eq!(ast, expected);
}

#[test]
fn raise() {
    let stream = Lexer::new("raise\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::Raise { exc: None, cause: None }
        ]
    };
    assert_eq!(ast, expected);
}

#[test]
fn raise_exc() {
    let stream = Lexer::new("raise Exception(\"a\")\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::Raise {
                exc: Some(Expression::Call {
                    func: Box::new(Expression:: Name {
                        id: String::from("Exception"),
                        ctx: ExprContext::Load
                    }),
                    args: vec![Expression::Str { s: String::from("a") }],
                    keywords: vec![]
                }),
                cause: None
            }
        ]
    };
    assert_eq!(ast, expected);
}

#[test]
fn raise_exc_from_cause() {
    let stream = Lexer::new("raise Exception(\"a\") from Exception(\"b\")\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::Raise {
                exc: Some(Expression::Call {
                    func: Box::new(Expression:: Name {
                        id: String::from("Exception"),
                        ctx: ExprContext::Load
                    }),
                    args: vec![Expression::Str { s: String::from("a") }],
                    keywords: vec![]
                }),
                cause: Some(Expression::Call {
                    func: Box::new(Expression:: Name {
                        id: String::from("Exception"),
                        ctx: ExprContext::Load
                    }),
                    args: vec![Expression::Str { s: String::from("b") }],
                    keywords: vec![]
                }),
            }
        ]
    };
    assert_eq!(ast, expected);
}

#[test]
fn dict_creation() {
    let stream = Lexer::new("{}\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::Expr {
                value: Expression::Dict {
                    keys: vec![],
                    values: vec![]
                }
            }
        ]
    };
    assert_eq!(ast, expected);

    let stream = Lexer::new("{a:b}\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::Expr {
                value: Expression::Dict {
                    keys: vec![
                        Expression::Name { id: String::from("a"),
                            ctx: ExprContext::Load }
                    ],
                    values: vec![
                        Expression::Name { id: String::from("b"),
                            ctx: ExprContext::Load }
                    ]
                }
            }
        ]
    };
    assert_eq!(ast, expected);

    let stream = Lexer::new("{a:c, **x, b:d,}\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::Expr {
                value: Expression::Dict {
                    keys: vec![
                        Expression::Name { id: String::from("a"),
                            ctx: ExprContext::Load },
                        Expression::None,
                        Expression::Name { id: String::from("b"),
                            ctx: ExprContext::Load }
                    ],
                    values: vec![
                        Expression::Name { id: String::from("c"),
                            ctx: ExprContext::Load },
                        Expression::Name { id: String::from("x"),
                            ctx: ExprContext::Load },
                        Expression::Name { id: String::from("d"),
                            ctx: ExprContext::Load }
                    ]
                }
            }
        ]
    };
    assert_eq!(ast, expected);

    let stream = Lexer::new("{a:c, **x, b:d}\n");
    let ast = parser::parse_start_symbol(stream);
    assert_eq!(ast, expected);
}

#[test]
fn list_creation() {
    let stream = Lexer::new("[]\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::Expr {
                value: Expression::List {
                    elts: vec![],
                    ctx: ExprContext::Load
                }
            }
        ]
    };
    assert_eq!(ast, expected);

    let stream = Lexer::new("[a,*b,]\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::Expr {
                value: Expression::List {
                    elts: vec![
                        Expression::Name { id: String::from("a"),
                            ctx: ExprContext::Load },
                        Expression::Starred {
                            value: Box::new(
                                Expression::Name { id: String::from("b"),
                                    ctx: ExprContext::Load },
                            ),
                            ctx: ExprContext::Load
                        }
                    ],
                    ctx: ExprContext::Load
                }
            }
        ]
    };
    assert_eq!(ast, expected);

    let stream = Lexer::new("[a,*b]\n");
    let ast = parser::parse_start_symbol(stream);
    assert_eq!(ast, expected);
}

#[test]
fn set_creation() {
    let stream = Lexer::new("{a}\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::Expr {
                value: Expression::Set {
                    elts: vec![
                        Expression::Name { id: String::from("a"),
                            ctx: ExprContext::Load }
                    ],
                }
            }
        ]
    };
    assert_eq!(ast, expected);

    let stream = Lexer::new("{a, *b,}\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::Expr {
                value: Expression::Set {
                    elts: vec![
                        Expression::Name { id: String::from("a"),
                            ctx: ExprContext::Load },
                        Expression::Starred {
                            value: Box::new(
                                Expression::Name { id: String::from("b"),
                                    ctx: ExprContext::Load },
                            ),
                            ctx: ExprContext::Load
                        }
                    ]
                }
            }
        ]
    };
    assert_eq!(ast, expected);

    let stream = Lexer::new("{a, *b}\n");
    let ast = parser::parse_start_symbol(stream);
    assert_eq!(ast, expected);
}

#[test]
fn list_comprehension() {
    let stream = Lexer::new("[a for x in y]\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::Expr {
                value: Expression::ListComp {
                    elt: Box::new(
                        Expression::Name { id: String::from("a"),
                            ctx: ExprContext::Load },
                    ),
                    generators: vec![
                        Comprehension::Comprehension {
                            target: Expression::Name { id: String::from("x"),
                                ctx: ExprContext::Load },
                            iter: Expression::Name { id: String::from("y"),
                                ctx: ExprContext::Load },
                            ifs: vec![]
                        }
                    ]
                }
            }
        ]
    };
    assert_eq!(ast, expected);

    let stream = Lexer::new("[a for x in y for g in q if True]\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::Expr {
                value: Expression::ListComp {
                    elt: Box::new(
                        Expression::Name { id: String::from("a"),
                            ctx: ExprContext::Load },
                    ),
                    generators: vec![
                        Comprehension::Comprehension {
                            target: Expression::Name { id: String::from("x"),
                                ctx: ExprContext::Load },
                            iter: Expression::Name { id: String::from("y"),
                                ctx: ExprContext::Load },
                            ifs: vec![]
                        },
                        Comprehension::Comprehension {
                            target: Expression::Name { id: String::from("g"),
                                ctx: ExprContext::Load },
                            iter: Expression::Name { id: String::from("q"),
                                ctx: ExprContext::Load },
                            ifs: vec![
                                Expression::NameConstant {
                                    value: Singleton::True
                                }
                            ]
                        }
                    ]
                }
            }
        ]
    };
    assert_eq!(ast, expected);
}

// TODO Update with proper contexts, that goes for all tests
#[test]
fn set_comprehension() {
    let stream = Lexer::new("{a for x in y}\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::Expr {
                value: Expression::SetComp {
                    elt: Box::new(
                        Expression::Name { id: String::from("a"),
                            ctx: ExprContext::Load },
                    ),
                    generators: vec![
                        Comprehension::Comprehension {
                            target: Expression::Name { id: String::from("x"),
                                ctx: ExprContext::Load },
                            iter: Expression::Name { id: String::from("y"),
                                ctx: ExprContext::Load },
                            ifs: vec![]
                        }
                    ]
                }
            }
        ]
    };
    assert_eq!(ast, expected);

    let stream = Lexer::new("{a for x in y for g in q if True}\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::Expr {
                value: Expression::SetComp {
                    elt: Box::new(
                        Expression::Name { id: String::from("a"),
                            ctx: ExprContext::Load },
                    ),
                    generators: vec![
                        Comprehension::Comprehension {
                            target: Expression::Name { id: String::from("x"),
                                ctx: ExprContext::Load },
                            iter: Expression::Name { id: String::from("y"),
                                ctx: ExprContext::Load },
                            ifs: vec![]
                        },
                        Comprehension::Comprehension {
                            target: Expression::Name { id: String::from("g"),
                                ctx: ExprContext::Load },
                            iter: Expression::Name { id: String::from("q"),
                                ctx: ExprContext::Load },
                            ifs: vec![
                                Expression::NameConstant {
                                    value: Singleton::True
                                }
                            ]
                        }
                    ]
                }
            }
        ]
    };
    assert_eq!(ast, expected);
}

#[test]
fn dict_comprehension() {
    let stream = Lexer::new("{a:b for x in y}\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::Expr {
                value: Expression::DictComp {
                    key: Box::new(
                        Expression::Name { id: String::from("a"),
                            ctx: ExprContext::Load },
                    ),
                    value: Box::new(
                        Expression::Name { id: String::from("b"),
                            ctx: ExprContext::Load },
                    ),
                    generators: vec![
                        Comprehension::Comprehension {
                            target: Expression::Name { id: String::from("x"),
                                ctx: ExprContext::Load },
                            iter: Expression::Name { id: String::from("y"),
                                ctx: ExprContext::Load },
                            ifs: vec![]
                        }
                    ]
                }
            }
        ]
    };
    assert_eq!(ast, expected);
}

#[test]
fn assignment() {
    let stream = Lexer::new("a = 3\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::Assign {
                targets: vec![
                    Expression::Name { id: String::from("a"),
                        ctx: ExprContext::Load }
                ],
                value: Expression::Num {
                    n: Number::DecInteger(String::from("3"))
                }
            }
        ]
    };
    assert_eq!(ast, expected);

    let stream = Lexer::new("a = yield\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::Assign {
                targets: vec![
                    Expression::Name { id: String::from("a"),
                        ctx: ExprContext::Load }
                ],
                value: Expression::Yield {
                    value: None
                }
            }
        ]
    };
    assert_eq!(ast, expected);

    let stream = Lexer::new("a = b = c = d = 3\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::Assign {
                targets: vec![
                    Expression::Name { id: String::from("a"),
                        ctx: ExprContext::Load },
                    Expression::Name { id: String::from("b"),
                        ctx: ExprContext::Load },
                    Expression::Name { id: String::from("c"),
                        ctx: ExprContext::Load },
                    Expression::Name { id: String::from("d"),
                        ctx: ExprContext::Load }
                ],
                value: Expression::Num {
                    n: Number::DecInteger(String::from("3"))
                }
            }
        ]
    };
    assert_eq!(ast, expected);
}

#[test]
fn annotated_assign() {
    let stream = Lexer::new("a : int\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::AnnAssign {
                target: Expression::Name { id: String::from("a"),
                    ctx: ExprContext::Load },
                annotation: Expression::Name { id: String::from("int"),
                    ctx: ExprContext::Load },
                value: None
            }
        ]
    };
    assert_eq!(ast, expected);

    let stream = Lexer::new("a : int = \"hi\"\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::AnnAssign {
                target: Expression::Name { id: String::from("a"),
                    ctx: ExprContext::Load },
                annotation: Expression::Name { id: String::from("int"),
                    ctx: ExprContext::Load },
                value: Some(Expression::Str { s: String::from("hi") })
            }
        ]
    };
    assert_eq!(ast, expected);
}

#[test]
fn augmented_assign() {
    let stream = Lexer::new("a += b; a -= b; a *= b; a @= b; a /= b; a %= b; \
        a &= b; a |= b; a ^= b; a <<= b; a >>= b; a **= b; a //= b\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::AugAssign {
                target: Expression::Name { id: String::from("a"),
                    ctx: ExprContext::Load },
                op: Operator::Add,
                value: Expression::Name { id: String::from("b"),
                    ctx: ExprContext::Load },
            },
            Statement::AugAssign {
                target: Expression::Name { id: String::from("a"),
                    ctx: ExprContext::Load },
                op: Operator::Sub,
                value: Expression::Name { id: String::from("b"),
                    ctx: ExprContext::Load },
            },
            Statement::AugAssign {
                target: Expression::Name { id: String::from("a"),
                    ctx: ExprContext::Load },
                op: Operator::Mult,
                value: Expression::Name { id: String::from("b"),
                    ctx: ExprContext::Load },
            },
            Statement::AugAssign {
                target: Expression::Name { id: String::from("a"),
                    ctx: ExprContext::Load },
                op: Operator::MatMult,
                value: Expression::Name { id: String::from("b"),
                    ctx: ExprContext::Load },
            },
            Statement::AugAssign {
                target: Expression::Name { id: String::from("a"),
                    ctx: ExprContext::Load },
                op: Operator::Div,
                value: Expression::Name { id: String::from("b"),
                    ctx: ExprContext::Load },
            },
            Statement::AugAssign {
                target: Expression::Name { id: String::from("a"),
                    ctx: ExprContext::Load },
                op: Operator::Mod,
                value: Expression::Name { id: String::from("b"),
                    ctx: ExprContext::Load },
            },
            Statement::AugAssign {
                target: Expression::Name { id: String::from("a"),
                    ctx: ExprContext::Load },
                op: Operator::BitAnd,
                value: Expression::Name { id: String::from("b"),
                    ctx: ExprContext::Load },
            },
            Statement::AugAssign {
                target: Expression::Name { id: String::from("a"),
                    ctx: ExprContext::Load },
                op: Operator::BitOr,
                value: Expression::Name { id: String::from("b"),
                    ctx: ExprContext::Load },
            },
            Statement::AugAssign {
                target: Expression::Name { id: String::from("a"),
                    ctx: ExprContext::Load },
                op: Operator::BitXor,
                value: Expression::Name { id: String::from("b"),
                    ctx: ExprContext::Load },
            },
            Statement::AugAssign {
                target: Expression::Name { id: String::from("a"),
                    ctx: ExprContext::Load },
                op: Operator::LShift,
                value: Expression::Name { id: String::from("b"),
                    ctx: ExprContext::Load },
            },
            Statement::AugAssign {
                target: Expression::Name { id: String::from("a"),
                    ctx: ExprContext::Load },
                op: Operator::RShift,
                value: Expression::Name { id: String::from("b"),
                    ctx: ExprContext::Load },
            },
            Statement::AugAssign {
                target: Expression::Name { id: String::from("a"),
                    ctx: ExprContext::Load },
                op: Operator::Pow,
                value: Expression::Name { id: String::from("b"),
                    ctx: ExprContext::Load },
            },
            Statement::AugAssign {
                target: Expression::Name { id: String::from("a"),
                    ctx: ExprContext::Load },
                op: Operator::FloorDiv,
                value: Expression::Name { id: String::from("b"),
                    ctx: ExprContext::Load },
            }
        ]
    };
    assert_eq!(ast, expected);
}

#[test]
fn assert() {
    let stream = Lexer::new("assert condition, \"message\"\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::Assert {
                test: Expression::Name {
                    id: String::from("condition"),
                    ctx: ExprContext::Load
                },
                msg: Some(Expression::Str { s: String::from("message") })
            }
        ]
    };
    assert_eq!(ast, expected);

    let stream = Lexer::new("assert condition\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::Assert {
                test: Expression::Name {
                    id: String::from("condition"),
                    ctx: ExprContext::Load
                },
                msg: None
            }
        ]
    };
    assert_eq!(ast, expected);
}

#[test]
fn import() {
    let stream = Lexer::new("import mod\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::Import {
                names: vec![
                    Alias::Alias { name: String::from("mod"), asname: None }
                ]
            }
        ]
    };
    assert_eq!(ast, expected);

    let stream = Lexer::new("import mod1.a.b as m, mod2\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::Import {
                names: vec![
                    Alias::Alias {
                        name: String::from("mod1.a.b"),
                        asname: Some(String::from("m"))
                    },
                    Alias::Alias { name: String::from("mod2"), asname: None }
                ]
            }
        ]
    };
    assert_eq!(ast, expected);
}

#[test]
fn import_from() {
    let stream = Lexer::new("from mod import *\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::ImportFrom {
                module: Some(String::from("mod")),
                names: vec![
                    Alias::Alias { name: String::from("*"), asname: None }
                ],
                level: 0
            }
        ]
    };
    assert_eq!(ast, expected);

    let stream = Lexer::new("from .... mod import a,b,c as g\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::ImportFrom {
                module: Some(String::from("mod")),
                names: vec![
                    Alias::Alias { name: String::from("a"), asname: None },
                    Alias::Alias { name: String::from("b"), asname: None },
                    Alias::Alias {
                        name: String::from("c"),
                        asname: Some(String::from("g"))
                    }
                ],
                level: 4
            }
        ]
    };
    assert_eq!(ast, expected);

    let stream = Lexer::new("from .... mod import (a,b,c as g,)\n");
    let ast = parser::parse_start_symbol(stream);
    assert_eq!(ast, expected);
}

#[test]
fn if_statement() {
    let stream = Lexer::new("if a:\n   x;y;z\n   x = 1\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::If {
                test: Expression::Name {
                    id: String::from("a"), ctx: ExprContext::Load },
                body: vec![
                    Statement::Expr { value: Expression::Name {
                        id: String::from("x"), ctx: ExprContext::Load
                    }},
                    Statement::Expr { value: Expression::Name {
                        id: String::from("y"), ctx: ExprContext::Load
                    }},
                    Statement::Expr { value: Expression::Name {
                        id: String::from("z"), ctx: ExprContext::Load
                    }},
                    Statement::Assign {
                        targets: vec![
                            Expression::Name {
                                id: String::from("x"), ctx: ExprContext::Load }
                        ],
                        value: Expression::Num {
                            n: Number::DecInteger(String::from("1")) }
                    }
                ],
                orelse: vec![]
            }
        ]
    };
    assert_eq!(ast, expected);

    let stream = Lexer::new("if a:\n   x;y;z\n   x = 1\nelif b:\n   func()\n\
        else:\n   pass\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::If {
                test: Expression::Name {
                    id: String::from("a"), ctx: ExprContext::Load },
                body: vec![
                    Statement::Expr { value: Expression::Name {
                        id: String::from("x"), ctx: ExprContext::Load
                    }},
                    Statement::Expr { value: Expression::Name {
                        id: String::from("y"), ctx: ExprContext::Load
                    }},
                    Statement::Expr { value: Expression::Name {
                        id: String::from("z"), ctx: ExprContext::Load
                    }},
                    Statement::Assign {
                        targets: vec![
                            Expression::Name {
                                id: String::from("x"), ctx: ExprContext::Load }
                        ],
                        value: Expression::Num {
                            n: Number::DecInteger(String::from("1")) }
                    }
                ],
                orelse: vec![
                    Statement::If {
                        test: Expression::Name {
                            id: String::from("b"), ctx: ExprContext::Load },
                        body: vec![
                            Statement::Expr {
                                value: Expression::Call {
                                    func: Box::new(Expression::Name {
                                        id: String::from("func"),
                                        ctx: ExprContext::Load
                                    }),
                                    args: vec![],
                                    keywords: vec![]
                                }
                            }
                        ],
                        orelse: vec![Statement::Pass]
                    }
                ]
            }
        ]
    };
    assert_eq!(ast, expected);
}

#[test]
fn while_statement() {
    let stream = Lexer::new("while True:\n   continue\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::While {
                test: Expression::NameConstant { value: Singleton::True },
                body: vec![Statement::Continue],
                orelse: vec![]
            }
        ]
    };
    assert_eq!(ast, expected);

    let stream = Lexer::new("while True:\n   continue\nelse:\n   pass\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::While {
                test: Expression::NameConstant { value: Singleton::True },
                body: vec![Statement::Continue],
                orelse: vec![Statement::Pass]
            }
        ]
    };
    assert_eq!(ast, expected);
}

// TODO update with proper context (Store)
#[test]
fn for_statment() {
    let stream = Lexer::new("for x in y:\n   pass\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::For {
                // TODO ctx should be a Store
                target: Expression::Name { id: String::from("x"),
                    ctx: ExprContext::Load },
                iter: Expression::Name { id: String::from("y"),
                    ctx: ExprContext::Load },
                body: vec![Statement::Pass],
                orelse: vec![]
            }
        ]
    };
    assert_eq!(ast, expected);

    let stream = Lexer::new("for x,y in a,b:\n   pass\nelse:\n   pass\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::For {
                target: Expression::Tuple {
                    elts: vec![
                        // TODO ctx's should be a Store
                        Expression::Name { id: String::from("x"),
                            ctx: ExprContext::Load },
                        Expression::Name { id: String::from("y"),
                            ctx: ExprContext::Load },
                    ],
                    ctx: ExprContext::Store
                },
                iter: Expression::Tuple {
                    elts: vec![
                        Expression::Name { id: String::from("a"),
                            ctx: ExprContext::Load },
                        Expression::Name { id: String::from("b"),
                            ctx: ExprContext::Load },
                    ],
                    ctx: ExprContext::Load
                },
                body: vec![Statement::Pass],
                orelse: vec![Statement::Pass]
            }
        ]
    };
    assert_eq!(ast, expected);

    let stream = Lexer::new("for x,y, in a,b,:\n   pass\nelse:\n   pass\n");
    let ast = parser::parse_start_symbol(stream);
    assert_eq!(ast, expected);
}

#[test]
fn with_statment() {
    let stream = Lexer::new("with a:\n   pass\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::With {
                items: vec![
                    WithItem::WithItem {
                        context_expr: Expression::Name {
                            id: String::from("a"), ctx: ExprContext::Load },
                        optional_vars: None
                    }
                ],
                body: vec![Statement::Pass]
            }
        ]
    };
    assert_eq!(ast, expected);

    let stream = Lexer::new("with a as x, b, c as z:\n   pass\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::With {
                items: vec![
                    WithItem::WithItem {
                        context_expr: Expression::Name {
                            id: String::from("a"), ctx: ExprContext::Load },
                        optional_vars: Some(Expression::Name {
                            id: String::from("x"), ctx: ExprContext::Load })
                    },
                    WithItem::WithItem {
                        context_expr: Expression::Name {
                            id: String::from("b"), ctx: ExprContext::Load },
                        optional_vars: None
                    },
                    WithItem::WithItem {
                        context_expr: Expression::Name {
                            id: String::from("c"), ctx: ExprContext::Load },
                        optional_vars: Some(Expression::Name {
                            id: String::from("z"), ctx: ExprContext::Load })
                    }
                ],
                body: vec![Statement::Pass]
            }
        ]
    };
    assert_eq!(ast, expected);
}

#[test]
fn try_statment() {
    let stream = Lexer::new("try:\n   x\nfinally:\n   fin\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::Try {
                body: vec![
                    Statement::Expr {
                        value: Expression::Name { id: String::from("x"),
                            ctx: ExprContext::Load }
                    }
                ],
                handlers: vec![],
                orelse: vec![],
                finalbody: vec![
                    Statement::Expr {
                        value: Expression::Name { id: String::from("fin"),
                            ctx: ExprContext::Load }
                    }
                ]
            }
        ]
    };
    assert_eq!(ast, expected);

    let stream = Lexer::new("try:\n   x\nexcept Error as e:\n   y\n\
        except NewError as e:\n   z\nelse:\n   pass\nfinally:\n   fin\n");
    let ast = parser::parse_start_symbol(stream);

    let expected = Ast::Module {
        body: vec![
            Statement::Try {
                body: vec![
                    Statement::Expr {
                        value: Expression::Name { id: String::from("x"),
                            ctx: ExprContext::Load }
                    }
                ],
                handlers: vec![
                    ExceptHandler::ExceptHandler {
                        etype: Some(Expression::Name {
                            id: String::from("Error"),
                            ctx: ExprContext::Load
                        }),
                        name: Some(String::from("e")),
                        body: vec![
                            Statement::Expr {
                                value: Expression::Name {
                                    id: String::from("y"),
                                    ctx: ExprContext::Load
                                }
                            }
                        ]
                    },
                    ExceptHandler::ExceptHandler {
                        etype: Some(Expression::Name {
                            id: String::from("NewError"),
                            ctx: ExprContext::Load
                        }),
                        name: Some(String::from("e")),
                        body: vec![
                            Statement::Expr {
                                value: Expression::Name {
                                    id: String::from("z"),
                                    ctx: ExprContext::Load
                                }
                            }
                        ]
                    }
                ],
                orelse: vec![
                    Statement::Pass
                ],
                finalbody: vec![
                    Statement::Expr {
                        value: Expression::Name { id: String::from("fin"),
                            ctx: ExprContext::Load }
                    }
                ]
            }
        ]
    };
    assert_eq!(ast, expected);
}
