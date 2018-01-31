mod util;
mod errors;

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use clap::ArgMatches;

use super::lexer::Lexer;
use super::parser;
use super::parser::ast::*;
use self::errors::CompilerError;

const INDENT: &str = "    ";

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

    compile_ast(&mut outfile, ast)
}

pub fn compile_ast(outfile: &mut File, ast: Ast) -> Result<(), CompilerError> {
    output_headers(outfile)?;
    output_main(outfile, &ast)
}

fn output_headers(outfile: &mut File) -> Result<(), CompilerError> {
    outfile.write_all("pub extern cannolib;\n".as_bytes()).unwrap();

    Ok(())
}

fn output_main(outfile: &mut File, ast: &Ast) -> Result<(), CompilerError> {
    let body = match *ast {
        Ast::Module { ref body } => body
    };

    outfile.write_all("fn main() {\n".as_bytes()).unwrap();

    for stmt in body.iter() {
        match *stmt {
            Statement::FunctionDef { .. } | Statement::ClassDef { .. } => (),
            _ => output_stmt(outfile, 1, stmt)?
        }
    }

    outfile.write_all("}\n".as_bytes()).unwrap();
    Ok(())
}

fn output_stmt(outfile: &mut File, indent: usize, stmt: &Statement)
    -> Result<(), CompilerError> {
    match *stmt {
        Statement::Expr { .. } => output_stmt_expr(outfile, indent, stmt),
        _ => unimplemented!()
    }
}

fn output_stmt_expr(outfile: &mut File, indent: usize, stmt: &Statement)
    -> Result<(), CompilerError> {
    let expr = match *stmt {
        Statement::Expr { ref value } => value,
        _ => unreachable!()
    };

    outfile.write_all(INDENT.repeat(indent).as_bytes()).unwrap();
    output_expr(outfile, expr)?;
    outfile.write_all("\n".as_bytes()).unwrap();
    Ok(())
}

fn output_expr(outfile: &mut File, expr: &Expression)
    -> Result<(), CompilerError> {
    Ok(())
}
