pub mod ast;
mod util;

use super::lexer::{Lexer, ResultToken};
use super::lexer::tokens::Token;
use self::ast::*;

fn parse_compound_stmt(opt: Option<(usize, ResultToken)>,
    mut stream: &mut Lexer) -> (Option<(usize, ResultToken)>, Statement) {
    unimplemented!()
}

fn parse_nonlocal_stmt(opt: Option<(usize, ResultToken)>,
    mut stream: &mut Lexer) -> (Option<(usize, ResultToken)>, SmallStatement) {
    let token = util::get_token(&opt);

    match token {
        Token::Identifier(s) => {
            let opt = stream.next();
            let token = util::get_token(&opt);

            match token {
                Token::Comma => {
                    let (opt, small_stmt) =
                        parse_nonlocal_stmt(stream.next(), stream);
                    let mut v = match small_stmt {
                        SmallStatement::NonlocalStatement(ids) => ids,
                        _ => panic!("invalid enum, found {:?}", small_stmt)
                    };

                    v.insert(0, s);
                    (opt, SmallStatement::NonlocalStatement(v))
                },
                _ => (opt, SmallStatement::NonlocalStatement(vec![s]))
            }
        }
        _ => panic!("expected 'identifier', found '{:?}'", token)
    }
}

fn parse_global_stmt(opt: Option<(usize, ResultToken)>, mut stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, SmallStatement) {
    let token = util::get_token(&opt);

    match token {
        Token::Identifier(s) => {
            let opt = stream.next();
            let token = util::get_token(&opt);

            match token {
                Token::Comma => {
                    let (opt, small_stmt) =
                        parse_global_stmt(stream.next(), stream);
                    let mut v = match small_stmt {
                        SmallStatement::GlobalStatement(ids) => ids,
                        _ => panic!("invalid enum, found {:?}", small_stmt)
                    };

                    v.insert(0, s);
                    (opt, SmallStatement::GlobalStatement(v))
                },
                _ => (opt, SmallStatement::GlobalStatement(vec![s]))
            }
        },
        _ => panic!("expected 'identifier', found '{:?}'", token)
    }
}

fn parse_small_stmt(opt: Option<(usize, ResultToken)>, mut stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, SmallStatement) {
    let token = util::get_token(&opt);

    match token {
        Token::Pass => (stream.next(), SmallStatement::PassStatement),
        Token::Global => parse_global_stmt(stream.next(), stream),
        Token::Nonlocal => parse_nonlocal_stmt(stream.next(), stream),
        _ => panic!("expected 'small_stmt', found {:?}", token)
    }
}

fn parse_simple_stmt(opt: Option<(usize, ResultToken)>, mut stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Statement) {
    let (opt, small_stmt) = parse_small_stmt(opt, &mut stream);
    let token = util::get_token(&opt);

    // TODO maybe use a peek here?
    match token {
        Token::Semi => {
            let opt = stream.next();
            let token = util::get_token(&opt);

            match token {
                Token::Newline => {
                    (stream.next(), Statement::SimpleStatement(vec![small_stmt]))
                },
                _ => {
                    let (opt, stmt) =
                        parse_simple_stmt(opt, stream);
                    let mut v = match stmt {
                        Statement::SimpleStatement(stmts) => stmts,
                        _ => panic!("invalid enum, found {:?}", stmt)
                    };

                    v.insert(0, small_stmt);
                    (opt, Statement::SimpleStatement(v))
                }
            }
        },
        Token::Newline => {
            (stream.next(), Statement::SimpleStatement(vec![small_stmt]))
        },
        bad_token => {
            panic!("expected ';' or '\\n', found '{:?}'", bad_token);
        }
    }
}

fn parse_stmt(opt: Option<(usize, ResultToken)>, mut stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Statement) {
    let token = util::get_token(&opt);

    // (simple_stmt | compound_stmt)
    if util::valid_simple_stmt(&token) {
        parse_simple_stmt(opt, &mut stream)
    } else {
        parse_compound_stmt(opt, &mut stream)
    }
}

fn parse_file_input(opt: Option<(usize, ResultToken)>,
    mut stream: &mut Lexer) -> (Option<(usize, ResultToken)>, Ast) {
    if opt.is_none() {
        return (opt, Ast::FileInput(vec![]));
    }

    let token = util::get_token(&opt);

    match token {
        Token::Newline => parse_file_input(stream.next(), &mut stream),
        _ => {
            let (opt, tree) = parse_stmt(opt, &mut stream);
            let (opt, Ast::FileInput(mut v)) =
                parse_file_input(opt, &mut stream);

            v.insert(0, tree);
            (opt, Ast::FileInput(v))
        }
    }
}

pub fn parse_start_symbol(mut stream: Lexer) -> Ast {
    let (next_token, ast) = parse_file_input(stream.next(), &mut stream);

    match next_token {
        Some(_) => panic!("expected 'EOF' found '{:?}'", next_token.unwrap()),
        None    => ast
    }
}
