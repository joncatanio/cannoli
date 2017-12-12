pub mod ast;

use super::lexer::Lexer;
use super::lexer::tokens::Token;
use self::ast::Ast;

/* REMOVE START */
use self::ast::{FileInput, Statement};
/* REMOVE END */

// Maybe return a result with a ParserError value
pub fn parse_file_input(mut stream: Lexer) -> Ast {
    match stream.next() {
        Some((int, result_token)) => println!("Token is {:?}", result_token),
        _ => unimplemented!()
    }

    // Temporary AST placeholder
    Ast::File(vec![FileInput::Statements(vec![Statement::SimpleStatement])])
}
