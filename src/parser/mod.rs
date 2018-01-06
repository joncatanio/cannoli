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
        let (opt, stmt) = parse_compound_stmt(opt, &mut stream);
        (opt, vec![stmt])
    }
}

// Both `parse_stmt` & `parse_stmts` return Vec structs, but `parse_stmts` calls
// `parse_stmt` recursively. If a compound_stmt is encountered then the Vec will
// be of size 1. Extending the Vec is a simple implementation of otherwise more
// complex logic.
fn parse_stmts(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Vec<Statement>) {
    if util::valid_stmt(&util::get_token(&opt)) {
        let (opt, mut stmt_vec) = parse_stmt(opt, stream);
        let (opt, stmts_vec) = parse_stmts(opt, stream);

        stmt_vec.extend(stmts_vec);
        (opt, stmt_vec)
    } else {
        (opt, vec![])
    }
}

fn parse_compound_stmt(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Statement) {
    match util::get_token(&opt) {
        Token::If    => parse_if_stmt(stream.next(), stream),
        Token::While => parse_while_stmt(stream.next(), stream),
        Token::For   => parse_for_stmt(stream.next(), stream),
        Token::Try   => parse_try_stmt(stream.next(), stream),
        Token::With  => parse_with_stmt(stream.next(), stream),
        _ => unimplemented!()
    }
}

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
        Token::Pass     => (stream.next(), Statement::Pass),
        Token::Global   => parse_global_stmt(stream.next(), stream),
        Token::Nonlocal => parse_nonlocal_stmt(stream.next(), stream),
        Token::Del      => parse_del_stmt(stream.next(), stream),
        Token::Assert   => parse_assert_stmt(stream.next(), stream),
        Token::Import   => parse_import_name(stream.next(), stream),
        Token::From     => parse_import_from(stream.next(), stream),
        ref token if util::valid_flow_stmt(&token) => {
            parse_flow_stmt(opt, stream)
        },
        _ => parse_expr_stmt(opt, stream)
    }
}

fn parse_expr_stmt(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Statement) {
    let (opt, expr) = parse_test_list_star_expr(opt, stream);

    match util::get_token(&opt) {
        Token::Colon => {
            let (opt, annotation, value) =
                parse_ann_assign(stream.next(), stream);

            (opt, Statement::AnnAssign { target: expr, annotation, value })
        },
        Token::Assign => {
            let (opt, mut targets, value) = parse_assign(stream.next(), stream);

            targets.insert(0, expr);
            (opt, Statement::Assign { targets, value })
        },
        ref token if util::valid_aug_assign(&token) => {
            let (opt, op) = parse_aug_assign(opt, stream);
            let (opt, value) = match util::get_token(&opt) {
                Token::Yield => parse_yield_expr(stream.next(), stream),
                _ => parse_test_list(opt, stream)
            };

            (opt, Statement::AugAssign { target: expr, op, value })
        },
        _ => (opt, Statement::Expr { value: expr })
    }
}

fn parse_assign(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Vec<Expression>, Expression) {
    let (opt, expr) = match util::get_token(&opt) {
        Token::Yield => parse_yield_expr(stream.next(), stream),
        _ => parse_test_list_star_expr(opt, stream)
    };

    match util::get_token(&opt) {
        Token::Assign => {
            let (opt, mut targets, value) = parse_assign(stream.next(), stream);

            targets.insert(0, expr);
            (opt, targets, value)
        },
        _ => (opt, vec![], expr)
    }
}

fn parse_ann_assign(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Expression, Option<Expression>) {
    let token = util::get_token(&opt);
    if !util::valid_test_expr(&token) {
        panic!("syntax error: invalid token {:?}", token)
    }

    let (opt, annotation) = parse_test_expr(opt, stream);
    match util::get_token(&opt) {
        Token::Assign => {
            let opt = stream.next();
            let token = util::get_token(&opt);
            if !util::valid_test_expr(&token) {
                panic!("syntax error: invalid token {:?}", token)
            }

            let (opt, value) = parse_test_expr(opt, stream);
            (opt, annotation, Some(value))
        },
        _ => (opt, annotation, None)
    }
}

fn parse_test_list_star_expr(opt: Option<(usize, ResultToken)>,
    stream: &mut Lexer) -> (Option<(usize, ResultToken)>, Expression) {
    let (opt, expr) = match util::get_token(&opt) {
        Token::Times => parse_star_expr(stream.next(), stream),
        _ => parse_test_expr(opt, stream)
    };

    match util::get_token(&opt) {
        Token::Comma => {
            let opt = stream.next();
            let token = util::get_token(&opt);

            if util::valid_test_star(&token) {
                let (opt, tup_expr) = parse_test_list_star_expr(opt, stream);
                let mut elts = match tup_expr {
                    Expression::Tuple { elts, .. } => elts,
                    expr => vec![expr]
                };

                elts.insert(0, expr);
                (opt, Expression::Tuple { elts, ctx: ExprContext::Load })
            } else {
                (opt, Expression::Tuple { elts: vec![expr],
                    ctx: ExprContext::Load })
            }
        },
        _ => (opt, expr)
    }
}

fn parse_aug_assign(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Operator) {
    let op = match util::get_token(&opt) {
        Token::AssignPlus        => Operator::Add,
        Token::AssignMinus       => Operator::Sub,
        Token::AssignTimes       => Operator::Mult,
        Token::AssignAt          => Operator::MatMult,
        Token::AssignDivide      => Operator::Div,
        Token::AssignMod         => Operator::Mod,
        Token::AssignBitAnd      => Operator::BitAnd,
        Token::AssignBitOr       => Operator::BitOr,
        Token::AssignBitXor      => Operator::BitXor,
        Token::AssignLshift      => Operator::LShift,
        Token::AssignRshift      => Operator::RShift,
        Token::AssignExponent    => Operator::Pow,
        Token::AssignDivideFloor => Operator::FloorDiv,
        t => panic!("parsing error: unexpected token for augassign, {:?}", t)
    };

    (stream.next(), op)
}

fn parse_del_stmt(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Statement) {
    if !util::valid_expr_list(&util::get_token(&opt)) {
        panic!("syntax error: invalid syntax after del keyword")
    }

    // TODO validate the targets to ensure deletion of proper things
    // TODO pass in context as Del
    let (opt, targets) = parse_expr_list(opt, stream);
    (opt, Statement::Delete { targets })
}

fn parse_flow_stmt(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Statement) {
    match util::get_token(&opt) {
        Token::Break    => (stream.next(), Statement::Break),
        Token::Continue => (stream.next(), Statement::Continue),
        Token::Return   => parse_return_stmt(stream.next(), stream),
        Token::Raise    => parse_raise_stmt(stream.next(), stream),
        Token::Yield    => parse_yield_stmt(stream.next(), stream),
        token => panic!("parser error: invalid flow_stmt token {:?}", token)
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

fn parse_yield_stmt(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Statement) {
    let (opt, value) = parse_yield_expr(opt, stream);
    (opt, Statement::Expr { value })
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

fn parse_import_name(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Statement) {
    let (opt, names) = parse_dotted_as_names(opt, stream);
    (opt, Statement::Import { names })
}

fn parse_import_from(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Statement) {
    let (opt, level) = parse_import_level(opt, stream);

    let (opt, module) = if util::valid_import_as_name(&util::get_token(&opt)) {
        let (opt, names) = parse_dotted_name(opt, stream);
        (opt, Some(names.join(".")))
    } else {
        (opt, None)
    };

    let (opt, names) = match util::get_token(&opt) {
        Token::Import => {
            let opt = stream.next();

            match util::get_token(&opt) {
                Token::Times => (stream.next(), vec![Alias::Alias {
                    name: String::from("*"), asname: None }]),
                Token::Lparen => {
                    let (opt, names) =
                        parse_import_as_names(stream.next(), stream);

                    match util::get_token(&opt) {
                        Token::Rparen => (stream.next(), names),
                        t => panic!("syntax error: expected ')', found {:?}", t)
                    }
                },
                _ => parse_import_as_names(opt, stream)
            }
        },
        t => panic!("syntax error: expected import, found {:?}", t)
    };

    (opt, Statement::ImportFrom { module, names, level })
}

pub fn parse_import_level(opt: Option<(usize, ResultToken)>,
    stream: &mut Lexer) -> (Option<(usize, ResultToken)>, usize) {
    match util::get_token(&opt) {
        Token::Dot => {
            let (opt, level) = parse_import_level(stream.next(), stream);
            (opt, level + 1)
        },
        Token::Ellipsis => {
            let (opt, level) = parse_import_level(stream.next(), stream);
            (opt, level + 3)
        },
        _ => (opt, 0)
    }
}

fn parse_import_as_name(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Alias) {
    let (opt, name) = match util::get_token(&opt) {
        Token::Identifier(s) => (stream.next(), s),
        t => panic!("syntax error: expeced identifier, found {:?}", t)
    };
    let (opt, asname) = match util::get_token(&opt) {
        Token::As => {
            let opt = stream.next();
            match util::get_token(&opt) {
                Token::Identifier(s) => (stream.next(), Some(s)),
                t => panic!("syntax error: expeced identifier, found {:?}", t)
            }
        },
        _ => (opt, None)
    };

    (opt, Alias::Alias { name, asname })
}

fn parse_dotted_as_name(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Alias) {
    let (opt, names) = parse_dotted_name(opt, stream);
    let name = names.join(".");

    match util::get_token(&opt) {
        Token::As => {
            let opt = stream.next();
            let (opt, asname) = match util::get_token(&opt) {
                Token::Identifier(s) => (stream.next(), s),
                t => panic!("syntax error: expected identifer, found {:?}", t)
            };

            (opt, Alias::Alias { name, asname: Some(asname) })
        },
        _ => (opt, Alias::Alias { name, asname: None })
    }
}

// CPython's parser reports an error when a trailing comma appears with no
// parentheses. ex: "from module import a,b,c," we might want to also error out
// but I don't see a reason for doing so at this moment in time.
fn parse_import_as_names(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Vec<Alias>) {
    if util::valid_import_as_name(&util::get_token(&opt)) {
        let (opt, alias) = parse_import_as_name(opt, stream);

        match util::get_token(&opt) {
            Token::Comma => {
                let (opt, mut aliases) =
                    parse_import_as_names(stream.next(), stream);

                aliases.insert(0, alias);
                (opt, aliases)
            },
            _ => (opt, vec![alias])
        }
    } else {
        (opt, vec![])
    }
}

fn parse_dotted_as_names(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Vec<Alias>) {
    let (opt, alias) = parse_dotted_as_name(opt, stream);

    match util::get_token(&opt) {
        Token::Comma => {
            let (opt, mut names) = parse_dotted_as_names(stream.next(), stream);

            names.insert(0, alias);
            (opt, names)
        },
        _ => (opt, vec![alias])
    }
}

// Returns a vec of strings, which can be joined with a '.', this is to
// anticipate any changes to parsing.
fn parse_dotted_name(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Vec<String>) {
    let (opt, name) = match util::get_token(&opt) {
        Token::Identifier(s) => (stream.next(), s),
        t => panic!("syntax error: expected identifier, found {:?}", t)
    };

    match util::get_token(&opt) {
        Token::Dot => {
            let (opt, mut names) = parse_dotted_name(stream.next(), stream);

            names.insert(0, name);
            (opt, names)
        },
        _ => (opt, vec![name])
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

fn parse_assert_stmt(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Statement) {
    if !util::valid_test_expr(&util::get_token(&opt)) {
        panic!("sytanx error: invalid syntax")
    }

    let (opt, test) = parse_test_expr(opt, stream);
    match util::get_token(&opt) {
        Token::Comma => {
            let opt = stream.next();

            if !util::valid_test_expr(&util::get_token(&opt)) {
                panic!("sytanx error: invalid syntax")
            }

            let (opt, msg) = parse_test_expr(opt, stream);
            (opt, Statement::Assert { test, msg: Some(msg) })
        },
        _ => (opt, Statement::Assert { test, msg: None })
    }
}

fn parse_if_stmt(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Statement) {
    let token = util::get_token(&opt);
    let (opt, test) = if util::valid_test_expr(&token) {
        parse_test_expr(opt, stream)
    } else {
        panic!("syntax error: invalid guard, found {:?}", token)
    };

    match util::get_token(&opt) {
        Token::Colon => {
            let (opt, body) = parse_suite(stream.next(), stream);

            // `compound_stmt` doesn't rely on an ending Newline token, we must
            // now check for EOF in some circumstance.
            if opt.is_none() {
                (opt, Statement::If { test, body, orelse: vec![] })
            } else {
                match util::get_token(&opt) {
                    Token::Elif => {
                        let (opt, stmt) = parse_if_stmt(stream.next(), stream);
                        (opt, Statement::If { test, body, orelse: vec![stmt] })
                    },
                    Token::Else => {
                        let opt = stream.next();

                        match util::get_token(&opt) {
                            Token::Colon => {
                                let (opt, orelse) =
                                    parse_suite(stream.next(), stream);
                                (opt, Statement::If { test, body, orelse })
                            },
                            t => panic!("syntax error: expected ':', \
                                found {:?}", t)
                        }
                    },
                    _ => (opt, Statement::If { test, body, orelse: vec![] })
                }
            }
        },
        t => panic!("syntax error: expected ':', found {:?}", t)
    }
}

fn parse_while_stmt(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Statement) {
    let token = util::get_token(&opt);
    let (opt, test) = if util::valid_test_expr(&token) {
        parse_test_expr(opt, stream)
    } else {
        panic!("syntax error: invalid guard, found {:?}", token)
    };

    match util::get_token(&opt) {
        Token::Colon => {
            let (opt, body) = parse_suite(stream.next(), stream);

            if opt.is_none() {
                (opt, Statement::While { test, body, orelse: vec![] })
            } else {
                match util::get_token(&opt) {
                    Token::Else => {
                        let opt = stream.next();

                        match util::get_token(&opt) {
                            Token::Colon => {
                                let (opt, orelse) =
                                    parse_suite(stream.next(), stream);
                                (opt, Statement::While { test, body, orelse })
                            },
                            t => panic!("syntax error: expected ':', \
                                found {:?}", t)
                        }
                    },
                    _ => (opt, Statement::While { test, body, orelse: vec![] })
                }
            }
        },
        t => panic!("syntax error: expected ':', found {:?}", t)
    }
}

fn parse_for_stmt(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Statement) {
    let (opt, mut expr_list) = parse_expr_list(opt, stream);
    let target = if expr_list.len() == 1 {
        expr_list.pop().unwrap()
    } else {
        Expression::Tuple { elts: expr_list, ctx: ExprContext::Store }
    };

    match util::get_token(&opt) {
        Token::In => {
            let (opt, iter) = parse_test_list(stream.next(), stream);

            match util::get_token(&opt) {
                Token::Colon => {
                    let (opt, body) = parse_suite(stream.next(), stream);

                    if opt.is_none() {
                        (opt, Statement::For {
                            target, iter, body, orelse: vec![] })
                    } else {
                        match util::get_token(&opt) {
                            Token::Else => {
                                let opt = stream.next();

                                match util::get_token(&opt) {
                                    Token::Colon => {
                                        let (opt, orelse) =
                                            parse_suite(stream.next(), stream);

                                        (opt, Statement::For {
                                            target, iter, body, orelse })
                                    },
                                    t => panic!("syntax error: expected ':', \
                                        found {:?}", t)
                                }
                            },
                            _ => (opt, Statement::For {
                                target, iter, body, orelse: vec![] })
                        }
                    }
                },
                t => panic!("syntax error: expected ':', found {:?}", t)
            }
        },
        t => panic!("syntax error: expected 'in', found {:?}", t)
    }
}

fn parse_try_stmt(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Statement) {
    let opt = match util::get_token(&opt) {
        Token::Colon => stream.next(),
        t => panic!("syntax error: expected ':', found {:?}", t)
    };
    let (opt, body) = parse_suite(opt, stream);

    match util::get_token(&opt) {
        Token::Except => {
            let (opt, handlers) = parse_except_clauses(opt, stream);
            let (opt, orelse) = if opt.is_none() {
                (opt, vec![])
            } else {
                match util::get_token(&opt) {
                    Token::Else => {
                        let opt = match util::get_token(&stream.next()) {
                            Token::Colon => stream.next(),
                            t => panic!("syntax error: expected ':', \
                                found {:?}", t)
                        };

                        parse_suite(opt, stream)
                    },
                    _ => (opt, vec![])
                }
            };
            let (opt, finalbody) = if opt.is_none() {
                (opt, vec![])
            } else {
                match util::get_token(&opt) {
                    Token::Finally => {
                        let opt = match util::get_token(&stream.next()) {
                            Token::Colon => stream.next(),
                            t => panic!("syntax error: expected ':', \
                                found {:?}", t)
                        };

                        parse_suite(opt, stream)
                    },
                    _ => (opt, vec![])
                }
            };

            (opt, Statement::Try { body, handlers, orelse, finalbody })
        },
        Token::Finally => {
            let opt = match util::get_token(&stream.next()) {
                Token::Colon => stream.next(),
                t => panic!("syntax error: expected ':', found {:?}", t)
            };
            let (opt, finalbody) = parse_suite(opt, stream);

            (opt, Statement::Try { body, handlers: vec![],
                orelse: vec![], finalbody })
        },
        _ => panic!("syntax error: invalid syntax")
    }
}

// The compiler can enforce the default exception being last
fn parse_except_clauses(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Vec<ExceptHandler>) {
    match util::get_token(&opt) {
        Token::Except => {
            let (opt, etype, name) = parse_except_clause(stream.next(), stream);
            let opt = match util::get_token(&opt) {
                Token::Colon => stream.next(),
                t => panic!("syntax error: expected ':', found {:?}", t)
            };

            let (opt, body) = parse_suite(opt, stream);
            let handler = ExceptHandler::ExceptHandler { etype, name, body };

            if opt.is_none() {
                (opt, vec![handler])
            } else {
                let (opt, mut handlers) =
                    parse_except_clauses(opt, stream);

                handlers.insert(0, handler);
                (opt, handlers)
            }
        },
        _ => (opt, vec![])
    }
}

fn parse_except_clause(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Option<Expression>, Option<String>) {
    if util::valid_test_expr(&util::get_token(&opt)) {
        let (opt, etype) = parse_test_expr(opt, stream);

        match util::get_token(&opt) {
            Token::As => {
                let opt = stream.next();

                match util::get_token(&opt) {
                    Token::Identifier(name) =>
                        (stream.next(), Some(etype), Some(name)),
                    t => panic!("syntax error: expected alias, found {:?}", t)
                }
            },
            _ => (opt, Some(etype), None)
        }
    } else {
        (opt, None, None)
    }
}

fn parse_with_stmt(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Statement) {
    let (opt, items) = parse_with_items(opt, stream);

    if items.is_empty() {
        panic!("syntax error: invalid syntax")
    }

    match util::get_token(&opt) {
        Token::Colon => {
            let (opt, body) = parse_suite(stream.next(), stream);

            (opt, Statement::With { items, body })
        },
        _ => panic!("syntax error: invalid syntax")
    }
}

fn parse_with_items(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Vec<WithItem>) {
    if util::valid_test_expr(&util::get_token(&opt)) {
        let (opt, item) = parse_with_item(opt, stream);

        match util::get_token(&opt) {
            Token::Comma => {
                let (opt, mut items) = parse_with_items(stream.next(), stream);

                items.insert(0, item);
                (opt, items)
            },
            _ => (opt, vec![item])
        }
    } else {
        (opt, vec![])
    }
}

fn parse_with_item(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, WithItem) {
    let (opt, context_expr) = parse_test_expr(opt, stream);

    match util::get_token(&opt) {
        Token::As => {
            let (opt, expr) = parse_expr(stream.next(), stream);
            (opt, WithItem::WithItem {context_expr, optional_vars: Some(expr)})
        },
        _ => (opt, WithItem::WithItem { context_expr, optional_vars: None })
    }
}

fn parse_suite(opt: Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Vec<Statement>) {
    match util::get_token(&opt) {
        Token::Newline => {
            let opt = stream.next();

            match util::get_token(&opt) {
                Token::Indent => {
                    let opt = stream.next();

                    if util::valid_stmt(&util::get_token(&opt)) {
                        let (opt, stmts) = parse_stmts(opt, stream);

                        match util::get_token(&opt) {
                            Token::Dedent => (stream.next(), stmts),
                            _ => panic!("indentation error: expeced dedent")
                        }
                    } else {
                        panic!("syntax error: expected statement in block")
                    }
                },
                _ => panic!("indentation error: expected indented block")
            }
        },
        token => {
            if util::valid_simple_stmt(&token) {
                let (opt, stmts) = parse_simple_stmt(opt, stream);
                (opt, stmts)
            } else {
                panic!("syntax error: invalid suite, found {:?}", token)
            }
        }
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
            let (opt, expr) = if util::valid_test_star(&token) {
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

    if util::valid_test_star(&token) {
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
// Expression might be pulled out.
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
