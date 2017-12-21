pub mod ast;
mod util;

use super::lexer::{Lexer, ResultToken};
use super::lexer::tokens::Token;
use self::ast::*;
use self::util::ArgType;

pub fn parse_start_symbol(mut stream: Lexer) -> Ast {
    let (next_token, ast) = parse_file_input(stream.next(), &mut stream);

    match next_token {
        Some(_) => panic!("expected 'EOF' found '{:?}'", next_token.unwrap()),
        None    => ast
    }
}

fn parse_file_input(opt: Option<(usize, ResultToken)>,
    mut stream: &mut Lexer) -> (Option<(usize, ResultToken)>, Ast) {
    if opt.is_none() {
        return (opt, Ast::Module { body: vec![] });
    }

    match util::get_token(&opt) {
        Token::Newline => parse_file_input(stream.next(), &mut stream),
        _ => {
            let (opt, mut stmt_vec) = parse_stmt(opt, &mut stream);
            let (opt, Ast::Module { body }) =
                parse_file_input(opt, &mut stream);

            stmt_vec.extend(body);
            (opt, Ast::Module { body: stmt_vec })
        }
    }
}

fn parse_stmt(opt: Option<(usize, ResultToken)>, mut stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Vec<Statement>) {
    let token = util::get_token(&opt);

    if util::valid_simple_stmt(&token) {
        parse_simple_stmt(opt, &mut stream)
    } else {
        /*
        let (opt, stmt) = parse_compound_stmt(opt, &mut stream);
        (opt, vec![stmt])
        */
        unimplemented!()
    }
}

/*
fn parse_compound_stmt(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Statement) {
    unimplemented!()
}
*/

fn parse_simple_stmt(opt: Option<(usize, ResultToken)>, mut stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Vec<Statement>) {
    let (opt, small_stmt) = parse_small_stmt(opt, &mut stream);

    // TODO maybe use a peek here?
    match util::get_token(&opt) {
        Token::Semi => {
            let opt = stream.next();

            match util::get_token(&opt) {
                Token::Newline => (stream.next(), vec![small_stmt]),
                _ => {
                    let (opt, mut stmts) = parse_simple_stmt(opt, stream);

                    stmts.insert(0, small_stmt);
                    (opt, stmts)
                }
            }
        },
        Token::Newline => {
            (stream.next(), vec![small_stmt])
        },
        bad_token => {
            panic!("expected ';' or '\\n', found '{:?}'", bad_token);
        }
    }
}

fn parse_small_stmt(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Statement) {
    match util::get_token(&opt) {
        Token::Pass => (stream.next(), Statement::Pass),
        Token::Global => parse_global_stmt(stream.next(), stream),
        Token::Nonlocal => parse_nonlocal_stmt(stream.next(), stream),
        ref token if util::valid_flow_stmt(&token) => {
            parse_flow_stmt(opt, stream)
        },
        _ => unimplemented!()
    }
}

fn parse_global_stmt(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Statement) {
    match util::get_token(&opt) {
        Token::Identifier(name) => {
            let opt = stream.next();

            match util::get_token(&opt) {
                Token::Comma => {
                    let (opt, stmt) = parse_global_stmt(stream.next(), stream);
                    let mut names = match stmt {
                        Statement::Global { names } => names,
                        _ => panic!("invalid enum, found {:?}", stmt)
                    };

                    names.insert(0, name);
                    (opt, Statement::Global { names })
                },
                _ => (opt, Statement::Global { names: vec![name] })
            }
        },
        token => panic!("expected 'identifier', found '{:?}'", token)
    }
}

fn parse_nonlocal_stmt(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Statement) {
    match util::get_token(&opt) {
        Token::Identifier(name) => {
            let opt = stream.next();

            match util::get_token(&opt) {
                Token::Comma => {
                    let (opt, stmt) =
                        parse_nonlocal_stmt(stream.next(), stream);
                    let mut names = match stmt {
                        Statement::Nonlocal { names } => names,
                        _ => panic!("invalid enum, found {:?}", stmt)
                    };

                    names.insert(0, name);
                    (opt, Statement::Nonlocal { names })
                },
                _ => (opt, Statement::Nonlocal { names: vec![name] })
            }
        }
        token => panic!("expected 'identifier', found '{:?}'", token)
    }
}

fn parse_flow_stmt(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Statement) {
    match util::get_token(&opt) {
        Token::Break    => (stream.next(), Statement::Break),
        Token::Continue => (stream.next(), Statement::Continue),
        Token::Return   => parse_return_stmt(stream.next(), stream),
        Token::Raise    => unimplemented!(),
        Token::Yield    => unimplemented!(), // Will return Statement::Expr
        _ => unimplemented!()
    }
}

fn parse_return_stmt(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Statement) {
    let token = util::get_token(&opt);

    if util::valid_test_expr(&token) {
        let (opt, test_list) = parse_test_list(opt, stream);
        (opt, Statement::Return { value: Some(test_list) })
    } else {
        (opt, Statement::Return { value: None })
    }
}

fn parse_test_list(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Expression) {
    let (opt, test_expr) = parse_test_expr(opt, stream);

    match util::get_token(&opt) {
        Token::Comma => {
            let opt = stream.next();
            let token = util::get_token(&opt);

            if util::valid_test_expr(&token) {
                let (opt, expr) = parse_test_list(opt, stream);
                let mut elts = match expr {
                    Expression::Tuple { elts, .. } => elts,
                    _ => vec![expr]
                };

                elts.insert(0, test_expr);
                (opt, Expression::Tuple { elts: elts, ctx: ExprContext::Load })
            } else {
                (
                    opt,
                    Expression::Tuple {
                        elts: vec![test_expr], ctx: ExprContext::Load
                    }
                )
            }
        },
        _ => (opt, test_expr)
    }
}

fn parse_test_expr(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Expression) {
    match util::get_token(&opt) {
        Token::Lambda => unimplemented!(),
        _ => {
            let (opt, expr) = parse_or_test(opt, stream);

            match util::get_token(&opt) {
                Token::If => {
                    let (opt, guard) = parse_or_test(stream.next(), stream);

                    match util::get_token(&opt) {
                        Token::Else => {
                            let (opt, else_expr) =
                                parse_test_expr(stream.next(), stream);

                            (
                                opt,
                                Expression::If {
                                    test: Box::new(guard),
                                    body: Box::new(expr),
                                    orelse: Box::new(else_expr)
                                }
                            )
                        },
                        token => panic!("expected 'else', found '{:?}'", token)
                    }
                },
                _ => (opt, expr)
            }
        }
    }
}

fn parse_or_test(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Expression) {
    let (opt, expr) = parse_and_test(opt, stream);

    match util::get_token(&opt) {
        Token::Or => {
            let (opt, mut values) = rec_parse_or_test(stream.next(), stream);

            values.insert(0, expr);
            (opt, Expression::BoolOp { op: BoolOperator::Or, values })
        },
        _ => (opt, expr)
    }
}

fn rec_parse_or_test(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Vec<Expression>) {
    let (opt, expr) = parse_and_test(opt, stream);

    match util::get_token(&opt) {
        Token::Or => {
            let (opt, mut values) = rec_parse_or_test(stream.next(), stream);

            values.insert(0, expr);
            (opt, values)
        },
        _ => (opt, vec![expr])
    }
}

fn parse_and_test(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Expression) {
    let (opt, expr) = parse_not_test(opt, stream);

    match util::get_token(&opt) {
        Token::And => {
            let (opt, mut values) = rec_parse_and_test(stream.next(), stream);

            values.insert(0, expr);
            (opt, Expression::BoolOp { op: BoolOperator::And, values })
        },
        _ => (opt, expr)
    }
}

fn rec_parse_and_test(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Vec<Expression>) {
    let (opt, expr) = parse_not_test(opt, stream);

    match util::get_token(&opt) {
        Token::And => {
            let (opt, mut values) = rec_parse_and_test(stream.next(), stream);

            values.insert(0, expr);
            (opt, values)
        },
        _ => (opt, vec![expr])
    }
}

fn parse_not_test(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Expression) {
    match util::get_token(&opt) {
        Token::Not => {
            let (opt, expr) = parse_not_test(stream.next(), stream);

            (opt, Expression::UnaryOp {
                op: UnaryOperator::Not, operand: Box::new(expr) })
        },
        _ => parse_comparison_expr(opt, stream)
    }
}

fn parse_comparison_expr(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Expression) {
    let (opt, expr) = parse_expr(opt, stream);
    let token = util::get_token(&opt);

    if util::valid_cmp_op(&token) {
        let (opt, ops, comparators) = rec_parse_comparison_expr(opt, stream);

        (opt, Expression::Compare { left: Box::new(expr), ops, comparators })
    } else {
        (opt, expr)
    }
}

fn rec_parse_comparison_expr(opt: Option<(usize, ResultToken)>,
    stream: &mut Lexer) -> (Option<(usize, ResultToken)>,
    Vec<CmpOperator>, Vec<Expression>) {
    let (opt, op) = util::get_cmp_op(&opt, stream).unwrap();
    let (opt, expr) = parse_expr(opt, stream);
    let token = util::get_token(&opt);

    if util::valid_cmp_op(&token) {
        let (opt, mut ops, mut comparators) =
            rec_parse_comparison_expr(opt, stream);

        ops.insert(0, op);
        comparators.insert(0, expr);
        (opt, ops, comparators)
    } else {
        (opt, vec![op], vec![expr])
    }
}

fn parse_expr(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Expression) {
    let (opt, expr) = parse_xor_expr(opt, stream);

    match util::get_token(&opt) {
        Token::BitOr => {
            let (opt, right_expr) = parse_expr(stream.next(), stream);

            (opt, Expression::BinOp { left: Box::new(expr),
                op: Operator::BitOr, right: Box::new(right_expr) })
        },
        _ => (opt, expr)
    }
}

fn parse_xor_expr(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Expression) {
    let (opt, expr) = parse_and_expr(opt, stream);

    match util::get_token(&opt) {
        Token::BitXor => {
            let (opt, right_expr) = parse_xor_expr(stream.next(), stream);

            (opt, Expression::BinOp { left: Box::new(expr),
                op: Operator::BitXor, right: Box::new(right_expr) })
        },
        _ => (opt, expr)
    }
}

fn parse_and_expr(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Expression) {
    let (opt, expr) = parse_shift_expr(opt, stream);

    match util::get_token(&opt) {
        Token::BitAnd => {
            let (opt, right_expr) = parse_and_expr(stream.next(), stream);

            (opt, Expression::BinOp { left: Box::new(expr),
                op: Operator::BitAnd, right: Box::new(right_expr) })
        },
        _ => (opt, expr)
    }
}

fn parse_shift_expr(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Expression) {
    let (opt, expr) = parse_arith_expr(opt, stream);

    match util::get_shift_op(&opt) {
        Some(op) => {
            let (opt, right_expr) = parse_shift_expr(stream.next(), stream);

            (opt, Expression::BinOp { left: Box::new(expr),
                op, right: Box::new(right_expr) })
        }
        None => (opt, expr)
    }
}

fn parse_arith_expr(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Expression) {
    let (opt, expr) = parse_term(opt, stream);

    match util::get_arith_op(&opt) {
        Some(op) => {
            let (opt, right_expr) = parse_arith_expr(stream.next(), stream);

            (opt, Expression::BinOp { left: Box::new(expr),
                op, right: Box::new(right_expr) })
        },
        None => (opt, expr)
    }
}

fn parse_term(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Expression) {
    let (opt, expr) = parse_factor(opt, stream);

    match util::get_term_op(&opt) {
        Some(op) => {
            let (opt, right_expr) = parse_term(stream.next(), stream);

            (opt, Expression::BinOp { left: Box::new(expr),
                op, right: Box::new(right_expr) })
        },
        None => (opt, expr)
    }
}

fn parse_factor(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Expression) {
    match util::get_factor_op(&opt) {
        Some(op) => {
            let (opt, operand) = parse_factor(stream.next(), stream);

            (opt, Expression::UnaryOp { op, operand: Box::new(operand) })
        },
        None => parse_power(opt, stream)
    }
}

fn parse_power(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Expression) {
    let (opt, expr) = parse_atom_expr(opt, stream);

    match util::get_token(&opt) {
        Token::Exponent => {
            let (opt, right_expr) = parse_factor(stream.next(), stream);

            (opt, Expression::BinOp { left: Box::new(expr),
                op: Operator::Pow, right: Box::new(right_expr) })
        },
        _ => (opt, expr)
    }
}

fn parse_atom_expr(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Expression) {
    let (opt, expr) = parse_atom(opt, stream);
    parse_atom_trailer(opt, expr, stream)
}

fn parse_atom_trailer(opt: Option<(usize, ResultToken)>, expr: Expression,
    stream: &mut Lexer) -> (Option<(usize, ResultToken)>, Expression) {
    match util::get_token(&opt) {
        Token::Lparen => {
            let (opt, args, keywords) = parse_arglist(stream.next(), stream);

            match util::get_token(&opt) {
                Token::Rparen => parse_atom_trailer(stream.next(),
                    Expression::Call { func: Box::new(expr), args, keywords },
                    stream),
                token => panic!("expected ')', found '{:?}'", token)
            }
        },
        Token::Lbracket => {
            unimplemented!()
        },
        Token::Dot => {
            unimplemented!()
        },
        _ => (opt, expr)
    }
}

fn parse_atom(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Expression) {
    match util::get_token(&opt) {
        Token::Lparen => {
            unimplemented!()
        },
        Token::Lbracket => {
            unimplemented!()
        },
        Token::Lbrace => {
            unimplemented!()
        },
        Token::Identifier(id) =>
            (stream.next(), Expression::Name { id, ctx: ExprContext::Load }),
        Token::DecInteger(n) =>
            (stream.next(), Expression::Num { n: Number::DecInteger(n) }),
        Token::BinInteger(n) =>
            (stream.next(), Expression::Num { n: Number::BinInteger(n) }),
        Token::OctInteger(n) =>
            (stream.next(), Expression::Num { n: Number::OctInteger(n) }),
        Token::HexInteger(n) =>
            (stream.next(), Expression::Num { n: Number::HexInteger(n) }),
        Token::Float(n) =>
            (stream.next(), Expression::Num { n: Number::Float(n) }),
        Token::Imaginary(n) =>
            (stream.next(), Expression::Num { n: Number::Imaginary(n) }),
        Token::String(s) => (stream.next(), Expression::Str { s }),
        Token::Ellipsis => (stream.next(), Expression::Ellipsis),
        Token::None =>
            (stream.next(),
                Expression::NameConstant { value: Singleton::None }),
        Token::True =>
            (stream.next(),
                Expression::NameConstant { value: Singleton::True }),
        Token::False =>
            (stream.next(),
                Expression::NameConstant { value: Singleton::False }),
        token => panic!("parsing error, found '{:?}'", token)
    }
}

fn parse_arglist(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Vec<Expression>, Vec<Keyword>) {
    let token = util::get_token(&opt);

    if util::valid_argument(&token) {
        rec_parse_arglist(opt, stream)
    } else {
        (opt, vec![], vec![])
    }
}

fn rec_parse_arglist(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Vec<Expression>, Vec<Keyword>) {
    let (opt, expr, arg, arg_type) = parse_argument(opt, stream);

    match util::get_token(&opt) {
        Token::Comma => {
            let opt = stream.next();
            let token = util::get_token(&opt);

            if util::valid_argument(&token) {
                let (opt, mut args, mut keywords) =
                    rec_parse_arglist(opt, stream);

                match arg_type {
                    ArgType::Positional => {
                        if keywords.is_empty() {
                            panic!("positional argument follows keyword \
                                    argument unpacking")
                        }
                        args.insert(0, expr)
                    },
                    ArgType::Keyword => keywords.insert(0,
                        Keyword::Keyword { arg, value: expr })
                }

                (opt, args, keywords)
            } else {
                // Trailing comma case
                match arg_type {
                    ArgType::Positional => (opt, vec![expr], vec![]),
                    ArgType::Keyword => (opt, vec![],
                        vec![Keyword::Keyword { arg, value: expr }])
                }
            }
        },
        _ => {
            match arg_type {
                ArgType::Positional => (opt, vec![expr], vec![]),
                ArgType::Keyword => (opt, vec![],
                    vec![Keyword::Keyword { arg, value: expr }])
            }
        }
    }
}

fn parse_argument(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Expression, Option<String>, ArgType) {
    match util::get_token(&opt) {
        Token::Exponent => {
            let (opt, expr) = parse_test_expr(stream.next(), stream);

            (opt, expr, None, ArgType::Keyword)
        },
        Token::Times => {
            let (opt, expr) = parse_test_expr(stream.next(), stream);

            (opt, Expression::Starred { value: Box::new(expr),
                ctx: ExprContext::Load }, None, ArgType::Positional)
        },
        _ => {
            let (opt, expr) = parse_test_expr(opt, stream);

            match util::get_token(&opt) {
                Token::Assign => {
                    let (opt, value) = parse_test_expr(stream.next(), stream);
                    let arg = match expr {
                        Expression::Name { id, .. } => id,
                        _ => panic!("keyword can't be expression")
                    };

                    (opt, value, Some(arg), ArgType::Keyword)
                },
                Token::For => {
                    // The Async token could come before For if we support it,
                    // in that case we may want to add a pattern to match
                    unimplemented!()
                },
                _ => (opt, expr, None, ArgType::Positional)
            }
        }
    }
}
