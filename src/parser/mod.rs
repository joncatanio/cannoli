pub mod ast;

use super::lexer::{Lexer, ResultToken};
use super::lexer::tokens::Token;
use self::ast::*;

// Maybe return a result with a ParserError value
/*
pub fn parse_file_input(mut stream: Lexer) -> Ast {
    let (next_token, ast) = parse_statement()
    match stream.next() {
        Some((int, result_token)) => println!("Token is {:?}", result_token),
        _ => unimplemented!()
    }

    // Temporary AST placeholder
    Ast::File(vec![FileInput::Statements(vec![Statement::SimpleStatement])])
}
*/

pub fn parse_stmt(opt: Option<(usize, ResultToken)>, mut stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, FileInput) {
    unimplemented!()
}

pub fn parse_file_input(opt: Option<(usize, ResultToken)>,
    mut stream: &mut Lexer) -> (Option<(usize, ResultToken)>, Ast) {
    if opt.is_some() {
        let (val, result_token) = opt.clone().unwrap();
        let token = result_token.clone().unwrap();

        match token {
            Token::Newline => parse_file_input(stream.next(), &mut stream),
            _ => {
                let (next_opt1, tree) = parse_stmt(opt, &mut stream);
                let (next_opt2, Ast::File(mut v)) =
                    parse_file_input(next_opt1, &mut stream);

                v.insert(1, tree);
                (next_opt2, Ast::File(v))
            }
        }
    } else {
        (opt, Ast::File(vec![]))
    }
}

pub fn parse_start_symbol(mut stream: Lexer) -> Ast {
    let (next_token, ast) = parse_file_input(stream.next(), &mut stream);

    match next_token {
        Some(_) => panic!("expected 'EOF' found '{:?}'", next_token.unwrap()),
        None    => ast
    }
}
