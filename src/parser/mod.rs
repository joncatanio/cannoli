pub mod ast;

use super::lexer::Lexer;
use self::ast::Ast;

pub fn parse_file_input(mut stream: Lexer) -> Option<Ast> {
    println!("Parsing! var is {:?}", stream.next());
    None
}
