#[macro_use(lazy_static)]
extern crate lazy_static;
extern crate regex;
extern crate unicode_names;

pub mod lexer;
pub mod parser;
pub mod compiler;

use lexer::Lexer;

fn main() {
    println!("Welcome to the Cannoli Compiler!");

    let chars = "'abc' \"def\" \\\n'123'\n";
    let l = Lexer::new(chars);

    let ast = parser::parse_file_input(l);
    println!("AST: {:?}", ast);
    compiler::compile(ast);
}
