mod util;
mod errors;

use std::fs::{File, OpenOptions};
use std::io::Read;
use clap::ArgMatches;

use super::lexer::Lexer;
use super::parser;
use super::parser::ast::*;
use self::errors::CompilerError;

// TODO maybe change this back to taking an ast only and when an import is
// encounter spawn a thread that calls back into cannoli with the new filename
pub fn compile_file(file: &str, opt_args: Option<&ArgMatches>)
    -> Result<(), CompilerError> {
    let mut fp = File::open(file).expect("file not found");
    let mut contents = String::new();
    fp.read_to_string(&mut contents)
        .expect("error reading the file");

    // Tokenize and parse file contents
    let stream = Lexer::new(&contents);
    let result = parser::parse_start_symbol(stream);
    let ast = if result.is_err() {
        return Err(CompilerError::ParserError(format!("{:?}", result)));
    } else {
        result.unwrap()
    };

    // Manage arguments if present
    if let Some(args) = opt_args {
        if args.is_present("parse") {
            println!("AST: {:?}", ast);
            return Ok(())
        }
    }

    println!("AST: {:?}", ast);
    // Create output file and pass it to compile
    let filename = util::get_file_prefix(file);
    let result = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(format!("{}.rs", filename));
    let mut outfile = if result.is_err() {
        return Err(CompilerError::IOError(format!("{:?}", result)));
    } else {
        result.unwrap()
    };

    compile_ast(ast)
}

pub fn compile_ast(ast: Ast) -> Result<(), CompilerError> {
    unimplemented!()
}
