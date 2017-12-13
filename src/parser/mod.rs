pub mod ast;
mod util;

use super::lexer::{Lexer, ResultToken};
use super::lexer::tokens::Token;
use self::ast::*;

fn parse_compound_stmt(opt: Option<(usize, ResultToken)>,
    mut stream: &mut Lexer) -> (Option<(usize, ResultToken)>, Statement) {
    unimplemented!()
}

fn parse_small_stmt(opt: Option<(usize, ResultToken)>, mut stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, SmallStatement) {
    let token = util::get_token(&opt);

    match token {
        Token::Pass => (stream.next(), SmallStatement::PassStatement),
        _ => panic!("expected 'small_stmt', found {:?}", token)
    }
}

fn parse_simple_stmt(opt: Option<(usize, ResultToken)>, mut stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, Statement) {
    let (next_opt1, small_stmt) = parse_small_stmt(opt, &mut stream);
    let next_token1 = util::get_token(&next_opt1);

    // TODO maybe use a peek here?
    match next_token1 {
        Token::Semi => {
            let next_opt2 = stream.next();
            let next_token2 = util::get_token(&next_opt2);

            match next_token2 {
                Token::Newline => {
                    (stream.next(), Statement::SimpleStatement(vec![small_stmt]))
                },
                _ => {
                    let (next_opt3, stmt) =
                        parse_simple_stmt(next_opt2, stream);
                    let mut v = match stmt {
                        Statement::SimpleStatement(stmts) => stmts,
                        _ => panic!("invalid enum, found {:?}", stmt)
                    };

                    v.insert(0, small_stmt);
                    (next_opt3, Statement::SimpleStatement(v))
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
            let (next_opt1, tree) = parse_stmt(opt, &mut stream);
            let (next_opt2, Ast::FileInput(mut v)) =
                parse_file_input(next_opt1, &mut stream);

            v.insert(0, tree);
            (next_opt2, Ast::FileInput(v))
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
