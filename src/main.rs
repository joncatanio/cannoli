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

    let chars = "'\\007'\n'\\7'\n'\\175'\n'\\x07'\n'\\1750'\n";
    let l = Lexer::new(chars);

    let ast = parser::parse_file_input(l);
    compiler::compile(ast);
}
