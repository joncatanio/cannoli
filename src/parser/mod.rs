pub mod ast;
mod util;

use super::lexer::{Lexer, ResultToken};
use super::lexer::tokens::Token;
use self::ast::*;
use self::util::{ArgType, TLCompType};

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
        Token::Raise    => parse_raise_stmt(stream.next(), stream),
        Token::Yield    => parse_yield_stmt(stream.next(), stream),
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

fn parse_raise_stmt(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Statement) {
    if util::valid_test_expr(&util::get_token(&opt)) {
        let (opt, exc) = parse_test_expr(opt, stream);

        match util::get_token(&opt) {
            Token::From => {
                let opt = stream.next();

                if !util::valid_test_expr(&util::get_token(&opt)) {
                    panic!("syntax error: expected value after 'from'")
                }

                let (opt, cause) = parse_test_expr(opt, stream);
                (opt, Statement::Raise { exc: Some(exc), cause: Some(cause) })
            },
            _ => (opt, Statement::Raise { exc: Some(exc), cause: None })
        }
    } else {
        (opt, Statement::Raise { exc: None, cause: None })
    }
}

fn parse_yield_stmt(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Statement) {
    let (opt, value) = parse_yield_expr(opt, stream);
    (opt, Statement::Expr { value })
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

fn parse_test_nocond(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Expression) {
    match util::get_token(&opt) {
        Token::Lambda => unimplemented!(),
        _ => parse_or_test(opt, stream)
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

// Basically a wrapper for `parse_expr` that returns Expression::Starred,
// the check for an asterisk should be done prior to calling this function
fn parse_star_expr(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Expression) {
    if !util::valid_expr(&util::get_token(&opt)) {
        panic!("syntax error: expected valid expression after '*'")
    }

    let (opt, expr) = parse_expr(opt, stream);
    (opt, Expression::Starred { value: Box::new(expr), ctx: ExprContext::Load })
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

fn parse_atom(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Expression) {
    match util::get_token(&opt) {
        Token::Lparen => {
            let opt = stream.next();
            let token = util::get_token(&opt);
            let (opt, expr) = if util::valid_atom_paren(&token) {
                match token {
                    Token::Yield => parse_yield_expr(stream.next(), stream),
                    _ => parse_test_list_comp(opt, TLCompType::Tuple, stream)
                }
            } else {
                (opt, Expression::Tuple {elts: vec![], ctx: ExprContext::Load})
            };

            match util::get_token(&opt) {
                Token::Rparen => (stream.next(), expr),
                _ => panic!("syntax error: expected closing paren")
            }
        },
        Token::Lbracket => {
            let opt = stream.next();
            let token = util::get_token(&opt);
            let (opt, expr) = if util::valid_test_list_comp(&token) {
                parse_test_list_comp(opt, TLCompType::List, stream)
            } else {
                (opt, Expression::List { elts: vec![], ctx: ExprContext::Load })
            };

            match util::get_token(&opt) {
                Token::Rbracket => (stream.next(), expr),
                _ => panic!("syntax error: expected closing bracket")
            }
        },
        Token::Lbrace => {
            let opt = stream.next();
            let token = util::get_token(&opt);
            let (opt, expr) = if util::valid_dict_set_maker(&token) {
                parse_dict_set_maker(opt, stream)
            } else {
                // There is no empty set literal, default to dict
                (opt, Expression::Dict { keys: vec![], values: vec![] })
            };

            match util::get_token(&opt) {
                Token::Rbrace => (stream.next(), expr),
                _ => panic!("syntax error: expected closing brace")
            }
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

fn parse_test_list_comp(opt: Option<(usize, ResultToken)>, ctype: TLCompType,
    stream: &mut Lexer) -> (Option<(usize, ResultToken)>, Expression) {
    let (opt, expr) = match util::get_token(&opt) {
        Token::Times => parse_star_expr(stream.next(), stream),
        _ => parse_test_expr(opt, stream)
    };

    match util::get_token(&opt) {
        Token::Comma => {
            let (opt, mut elts) =
                rec_parse_test_list_comp(stream.next(), stream);

            elts.insert(0, expr);
            match ctype {
                TLCompType::Tuple =>
                    (opt, Expression::Tuple { elts, ctx: ExprContext::Load }),
                TLCompType::List  =>
                    (opt, Expression::List { elts, ctx: ExprContext::Load })
            }
        },
        Token::For => {
            match ctype {
                TLCompType::Tuple => parse_comp_for(stream.next(),
                    Expression::Generator { elt: Box::new(expr),
                    generators: vec![] }, stream),
                TLCompType::List  => parse_comp_for(stream.next(),
                    Expression::ListComp { elt: Box::new(expr),
                    generators: vec![] }, stream)
            }
        },
        _ => (opt, expr)
    }
}

// Gets the list of test/star expressions for a non-comprehension descent
fn rec_parse_test_list_comp(opt: Option<(usize, ResultToken)>,
    stream: &mut Lexer) -> (Option<(usize, ResultToken)>, Vec<Expression>) {
    let token = util::get_token(&opt);

    if util::valid_test_list_comp(&token) {
        let (opt, expr) = match token {
            Token::Times => parse_star_expr(stream.next(), stream),
            _ => parse_test_expr(opt, stream)
        };

        match util::get_token(&opt) {
            Token::Comma => {
                let (opt, mut exprs) =
                    rec_parse_test_list_comp(stream.next(), stream);

                exprs.insert(0, expr);
                (opt, exprs)
            },
            _ => (opt, vec![expr])
        }
    } else {
        (opt, vec![])
    }
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
            let (opt, slice) = parse_subscript_list(stream.next(), stream);

            match util::get_token_expect(&opt, Token::Rbracket) {
                Token::Rbracket => parse_atom_trailer(stream.next(),
                    Expression::Subscript {
                        value: Box::new(expr), slice: Box::new(slice),
                        ctx: ExprContext::Load
                    },
                    stream),
                token => panic!("expected ']', found '{:?}'", token)
            }
        },
        Token::Dot => {
            match util::get_token(&stream.next()) {
                Token::Identifier(attr) => parse_atom_trailer(stream.next(),
                    Expression::Attribute {
                        value: Box::new(expr), attr, ctx: ExprContext::Load
                    },
                    stream
                ),
                token => panic!("expected identifier, found '{:?}'", token)
            }
        },
        _ => (opt, expr)
    }
}

fn parse_subscript_list(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Slice) {
    let (opt, slice) = parse_subscript(opt, stream);
    // We need to keep track of a trailing comma and only one subscript.
    let trailing_comma = match util::get_token(&opt) {
        Token::Comma => true,
        _ => false
    };
    let opt = if trailing_comma { stream.next() } else { opt };
    let (opt, mut slices) = rec_parse_subscript_list(opt, stream);

    if slices.is_empty() {
        if trailing_comma {
            match slice {
                Slice::Slice { .. } => {
                    (opt, Slice::ExtSlice { dims: vec![slice] })
                },
                Slice::Index { value } => {
                    (opt, Slice::Index { value: Expression::Tuple {
                        elts: vec![value], ctx: ExprContext::Load } })
                },
                _ => panic!("parsing error: `parse_subscript_list`")
            }
        } else {
            (opt, slice)
        }
    } else {
        slices.insert(0, slice);

        // If a Slice::Slice is found then we need to create an ExtSlice
        // instead of a Index with a Tuple
        let mut contains_slice = false;
        for s in slices.iter() {
            match *s {
                Slice::Slice { .. } => contains_slice = true,
                _ => ()
            }
        };

        if contains_slice {
            (opt, Slice::ExtSlice { dims: slices })
        } else {
            // Consumes `slices` variable
            let expr_list = slices.into_iter().map(|s| {
                match s {
                    Slice::Index { value } => value,
                    _ => panic!("parser error: expected Index values only")
                }
            }).collect();

            (opt, Slice::Index { value: Expression::Tuple {
                elts: expr_list, ctx: ExprContext::Load } })
        }
    }
}

fn rec_parse_subscript_list(opt: Option<(usize, ResultToken)>,
    stream: &mut Lexer) -> (Option<(usize, ResultToken)>, Vec<Slice>) {
    if util::valid_subscript(&util::get_token(&opt)) {
        let (opt, slice) = parse_subscript(opt, stream);

        match util::get_token(&opt) {
            Token::Comma => {
                let (opt, mut slices) =
                    rec_parse_subscript_list(stream.next(), stream);

                slices.insert(0, slice);
                (opt, slices)
            },
            _ => (opt, vec![slice])
        }
    } else {
        (opt, vec![])
    }
}

fn parse_subscript(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Slice) {
    let token = util::get_token(&opt);
    let (opt, lower) = if util::valid_test_expr(&token) {
        let (opt, expr) = parse_test_expr(opt, stream);
        (opt, Some(expr))
    } else {
        (opt, None)
    };

    match util::get_token(&opt) {
        Token::Colon => {
            let opt = stream.next();
            let token = util::get_token(&opt);
            let (opt, upper) = if util::valid_test_expr(&token) {
                let (opt, expr) = parse_test_expr(opt, stream);
                (opt, Some(expr))
            } else {
                (opt, None)
            };
            let (opt, step) = parse_sliceop(opt, stream);

            (opt, Slice::Slice { lower, upper, step })
        },
        _ => (opt, Slice::Index { value: lower.unwrap() })
    }
}

fn parse_sliceop(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Option<Expression>) {
    match util::get_token(&opt) {
        Token::Colon => {
            let opt = stream.next();
            let token = util::get_token(&opt);

            if util::valid_test_expr(&token) {
                let (opt, expr) = parse_test_expr(opt, stream);
                (opt, Some(expr))
            } else {
                (opt, None)
            }
        },
        _ => (opt, None)
    }
}

// Returns a Vec since there are multiple Expression values that wrap the
// expression list. If a Vec of size one is returned, the contained
// Expression should be pulled out.
fn parse_expr_list(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Vec<Expression>) {
    let (opt, expr) = match util::get_token(&opt) {
        Token::Times => parse_star_expr(stream.next(), stream),
        _ => parse_expr(opt, stream)
    };

    match util::get_token(&opt) {
        Token::Comma => {
            let opt = stream.next();
            let token = util::get_token(&opt);

            if util::valid_expr_list(&token) {
                let (opt, mut exprs) = parse_expr_list(opt, stream);

                exprs.insert(0, expr);
                (opt, exprs)
            } else {
                // Trailing comma case
                (opt, vec![expr])
            }
        },
        _ => (opt, vec![expr])
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

fn parse_dict_set_maker(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Expression) {
    // Get the first expression value and determine if a dict|set is being made
    let (opt, expr, value, is_dict) = match util::get_token(&opt) {
        Token::Exponent => {
            let (opt, expr) = parse_expr(stream.next(), stream);
            (opt, Expression::None, Some(expr), true)
        },
        Token::Times => {
            let (opt, expr) = parse_star_expr(stream.next(), stream);
            (opt, expr, None, false)
        }
        _ => {
            let (opt, expr) = parse_test_expr(opt, stream);

            match util::get_token(&opt) {
                Token::Colon => {
                    let opt = stream.next();
                    let token = util::get_token(&opt);

                    if !util::valid_test_expr(&token) {
                        panic!("syntax error: expected right hand expression \
                                in dictionary creation, found {:?}", token)
                    }

                    let (opt, value) = parse_test_expr(opt, stream);
                    (opt, expr, Some(value), true)
                },
                _ => (opt, expr, None, false)
            }
        }
    };

    if is_dict {
        parse_dict_maker(opt, expr, value.unwrap(), stream)
    } else {
        parse_set_maker(opt, expr, stream)
    }
}

fn parse_dict_maker(opt: Option<(usize, ResultToken)>, key: Expression,
    value: Expression, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Expression) {
    match util::get_token(&opt) {
        Token::Comma => {
            let (opt, mut keys, mut values) =
                rec_parse_dict_maker(stream.next(), stream);

            keys.insert(0, key);
            values.insert(0, value);
            (opt, Expression::Dict { keys, values })
        },
        Token::For => {
            // TODO error if dict unpacking was matched above
            parse_comp_for(stream.next(),
                Expression::DictComp { key: Box::new(key),
                value: Box::new(value), generators: vec![] }, stream)
        },
        _ => (opt, Expression::Dict {
            keys: vec![key], values: vec![value] })
    }
}

fn rec_parse_dict_maker(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Vec<Expression>, Vec<Expression>) {
    let token = util::get_token(&opt);

    if util::valid_dict_maker(&token) {
        let (opt, key, value) = match util::get_token(&opt) {
            Token::Exponent => {
                let (opt, expr) = parse_expr(stream.next(), stream);
                (opt, Expression::None, expr)
            },
            _ => {
                let (opt, key) = parse_test_expr(opt, stream);
                let token = util::get_token(&opt);

                match token {
                    Token::Colon => {
                        let opt = stream.next();
                        let token = util::get_token(&opt);

                        if !util::valid_test_expr(&token) {
                            panic!("syntax error: expected right \
                                    hand expression in dictionary creation, \
                                    found {:?}", token)
                        }

                        let (opt, value) = parse_test_expr(opt, stream);
                        (opt, key, value)
                    },
                    _ => panic!("syntax error: expected ':', found {:?}", token)
                }
            }
        };

        match util::get_token(&opt) {
            Token::Comma => {
                let (opt, mut keys, mut values) =
                    rec_parse_dict_maker(stream.next(), stream);

                keys.insert(0, key);
                values.insert(0, value);
                (opt, keys, values)
            },
            _ => (opt, vec![key], vec![value])
        }
    } else {
        (opt, vec![], vec![])
    }
}

fn parse_set_maker(opt: Option<(usize, ResultToken)>, expr: Expression,
    stream: &mut Lexer) -> (Option<(usize, ResultToken)>, Expression) {
    match util::get_token(&opt) {
        Token::Comma => {
            // Reuse the testlist comp builder since it's the same pattern
            let (opt, mut elts) =
                rec_parse_test_list_comp(stream.next(), stream);

            elts.insert(0, expr);
            (opt, Expression::Set { elts })
        },
        Token::For => {
            parse_comp_for(stream.next(),
                Expression::SetComp { elt: Box::new(expr),
                generators: vec![] }, stream)
        },
        _ => (opt, Expression::Set { elts: vec![expr] })
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
                    let (opt, expr) = parse_comp_for(stream.next(),
                        Expression::Generator { elt: Box::new(expr),
                        generators: vec![] }, stream);
                    (opt, expr, None, ArgType::Positional)
                },
                _ => (opt, expr, None, ArgType::Positional)
            }
        }
    }
}

fn parse_comp_iter(opt: Option<(usize, ResultToken)>, gc_expr: Expression,
    stream: &mut Lexer) -> (Option<(usize, ResultToken)>, Expression) {
    match util::get_token(&opt) {
        Token::For => parse_comp_for(stream.next(), gc_expr, stream),
        Token::If  => parse_comp_if(stream.next(), gc_expr, stream),
        _ => (opt, gc_expr)
    }
}

// Returns updated Generator/Comp, it's up to the caller to supply this method
// with a Expression::(Generator|*Comp) that will be "filled"
fn parse_comp_for(opt: Option<(usize, ResultToken)>, gc_expr: Expression,
    stream: &mut Lexer) -> (Option<(usize, ResultToken)>, Expression) {
    let (opt, mut expr_list) = parse_expr_list(opt, stream);
    let token = util::get_token(&opt);

    if token != Token::In {
        panic!("expected 'in' keyword, found '{:?}'", token);
    }
    let (opt, iter) = parse_or_test(stream.next(), stream);

    // Comprehensions deal with Tuples so if the size of the list is greater
    // than one element we make it a tuple. The `del` keyword expects an
    // exprlist in the grammar but doesn't expect a Tuple, therefore we let the
    // caller manage the Expression type.
    let target = if expr_list.len() == 1 {
        expr_list.pop().unwrap()
    } else {
        Expression::Tuple { elts: expr_list, ctx: ExprContext::Load }
    };
    let comp = Comprehension::Comprehension { target, iter, ifs: vec![] };

    match gc_expr {
        Expression::Generator { elt, mut generators } => {
            generators.push(comp);
            parse_comp_iter(opt,
                Expression::Generator { elt, generators }, stream)
        },
        Expression::ListComp { elt, mut generators }  => {
            generators.push(comp);
            parse_comp_iter(opt,
                Expression::ListComp { elt, generators }, stream)
        },
        Expression::SetComp { elt, mut generators }  => {
            generators.push(comp);
            parse_comp_iter(opt,
                Expression::SetComp { elt, generators }, stream)
        },
        Expression::DictComp { key, value, mut generators }  => {
            generators.push(comp);
            parse_comp_iter(opt,
                Expression::DictComp { key, value, generators }, stream)
        },
        _ => panic!("parsing error: expected gen/comp, found {:?}", gc_expr)
    }
}

// Modifies the most recent Comprehension within the generators
fn parse_comp_if(opt: Option<(usize, ResultToken)>, gc_expr: Expression,
    stream: &mut Lexer) -> (Option<(usize, ResultToken)>, Expression) {
    let (opt, expr) = parse_test_nocond(opt, stream);

    // TODO refactor this in a Rust-esque manner to limit repeated code
    match gc_expr {
        Expression::Generator { elt, mut generators } => {
            let (target, iter, mut ifs) = match generators.pop().unwrap() {
                Comprehension::Comprehension { target, iter, ifs } =>
                    (target, iter, ifs)
            };

            ifs.push(expr);
            generators.push(Comprehension::Comprehension { target, iter, ifs });
            parse_comp_iter(opt,
                Expression::Generator { elt, generators }, stream)
        },
        Expression::ListComp { elt, mut generators } => {
            let (target, iter, mut ifs) = match generators.pop().unwrap() {
                Comprehension::Comprehension { target, iter, ifs } =>
                    (target, iter, ifs)
            };

            ifs.push(expr);
            generators.push(Comprehension::Comprehension { target, iter, ifs });
            parse_comp_iter(opt,
                Expression::ListComp { elt, generators }, stream)
        },
        Expression::SetComp { elt, mut generators } => {
            let (target, iter, mut ifs) = match generators.pop().unwrap() {
                Comprehension::Comprehension { target, iter, ifs } =>
                    (target, iter, ifs)
            };

            ifs.push(expr);
            generators.push(Comprehension::Comprehension { target, iter, ifs });
            parse_comp_iter(opt,
                Expression::SetComp { elt, generators }, stream)
        },
        Expression::DictComp { key, value, mut generators } => {
            let (target, iter, mut ifs) = match generators.pop().unwrap() {
                Comprehension::Comprehension { target, iter, ifs } =>
                    (target, iter, ifs)
            };

            ifs.push(expr);
            generators.push(Comprehension::Comprehension { target, iter, ifs });
            parse_comp_iter(opt,
                Expression::DictComp { key, value, generators }, stream)
        },
        _ => panic!("parsing error: expected gen/comp, found {:?}", gc_expr)
    }
}

fn parse_yield_expr(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Expression) {
    if util::valid_yield_arg(&util::get_token(&opt)) {
        parse_yield_arg(opt, stream)
    } else {
        (opt, Expression::Yield { value: None })
    }
}

fn parse_yield_arg(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Expression) {
    match util::get_token(&opt) {
        Token::From => {
            let opt = stream.next();

            if !util::valid_test_expr(&util::get_token(&opt)) {
                panic!("syntax error: expected value after 'from'")
            }

            let (opt, expr) = parse_test_expr(opt, stream);
            (opt, Expression::YieldFrom { value: Box::new(expr) })
        },
        _ => {
            let (opt, expr) = parse_test_list(opt, stream);
            (opt, Expression::Yield { value: Some(Box::new(expr)) })
        }
    }
}
