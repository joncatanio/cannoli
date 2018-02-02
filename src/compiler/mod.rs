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
    outfile.write_all("extern crate cannolib;\n".as_bytes()).unwrap();

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

fn output_stmts(outfile: &mut File, indent: usize, stmts: &Vec<Statement>)
    -> Result<(), CompilerError> {
    for stmt in stmts.iter() {
        output_stmt(outfile, indent, stmt)?;
    }
    Ok(())
}

fn output_stmt(outfile: &mut File, indent: usize, stmt: &Statement)
    -> Result<(), CompilerError> {
    match *stmt {
        Statement::If { .. } => output_stmt_if(outfile, indent, stmt),
        Statement::Expr { .. } => output_stmt_expr(outfile, indent, stmt),
        _ => unimplemented!()
    }
}

fn output_stmt_if(outfile: &mut File, indent: usize, stmt: &Statement)
    -> Result<(), CompilerError> {
    let (test, body, orelse) = match *stmt {
        Statement::If { ref test, ref body, ref orelse } =>
            (test, body, orelse),
        _ => unreachable!()
    };

    // guard and decorators
    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write_all("if (".as_bytes()).unwrap();
    output_expr(outfile, test)?;
    outfile.write_all(").to_bool() {\n".as_bytes()).unwrap();

    // `then` body
    output_stmts(outfile, indent + 1, body)?;

    // closing decorator
    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write_all("}\n".as_bytes()).unwrap();
    Ok(())
}

fn output_stmt_expr(outfile: &mut File, indent: usize, stmt: &Statement)
    -> Result<(), CompilerError> {
    let expr = match *stmt {
        Statement::Expr { ref value } => value,
        _ => unreachable!()
    };

    outfile.write_all(INDENT.repeat(indent).as_bytes()).unwrap();
    output_expr(outfile, expr)?;
    outfile.write_all(";\n".as_bytes()).unwrap();
    Ok(())
}

fn output_expr(outfile: &mut File, expr: &Expression)
    -> Result<(), CompilerError> {
    match *expr {
        Expression::BinOp { .. } => output_expr_binop(outfile, expr),
        Expression::Compare { .. } => output_expr_cmp(outfile, expr),
        Expression::Num { ref n } => output_expr_num(outfile, n),
        Expression::NameConstant { ref value } =>
            output_expr_name_const(outfile, value),
        _ => unimplemented!()
    }
}

fn output_expr_binop(outfile: &mut File, expr: &Expression)
    -> Result<(), CompilerError> {
    let (left, op, right) = match *expr {
        Expression::BinOp { ref left, ref op, ref right } => (left, op, right),
        _ => unreachable!()
    };

    output_expr(outfile, left)?;
    outfile.write_all(" ".as_bytes()).unwrap();
    output_operator(outfile, op)?;
    outfile.write_all(" ".as_bytes()).unwrap();
    output_expr(outfile, right)?;
    Ok(())
}

fn output_expr_cmp(outfile: &mut File, expr: &Expression)
    -> Result<(), CompilerError> {
    unimplemented!()
}

fn output_expr_num(outfile: &mut File, num: &Number)
    -> Result<(), CompilerError> {
    let out_str = match *num {
        Number::DecInteger(ref s) => {
            format!("cannolib::Value::Number(\
                cannolib::NumericType::Integer({}))", s)
        },
        Number::BinInteger(ref s) => {
            format!("cannolib::Value::Number(\
                cannolib::NumericType::Integer({}))", s)
        },
        Number::OctInteger(ref s) => {
            format!("cannolib::Value::Number(\
                cannolib::NumericType::Integer({}))", s)
        },
        Number::HexInteger(ref s) => {
            format!("cannolib::Value::Number(\
                cannolib::NumericType::Integer({}))", s)
        },
        Number::Float(ref s) => {
            format!("cannolib::Value::Number(\
                cannolib::NumericType::Float({}))", s)
        },
        Number::Imaginary(_) => unimplemented!()
    };

    outfile.write_all(out_str.as_bytes()).unwrap();
    Ok(())
}

fn output_expr_name_const(outfile: &mut File, value: &Singleton)
    -> Result<(), CompilerError> {
    let out_str = match *value {
        Singleton::None  => format!("cannolib::Value::None"),
        Singleton::True  => format!("cannolib::Value::Bool(true)"),
        Singleton::False => format!("cannolib::Value::Bool(false)"),
    };

    outfile.write_all(out_str.as_bytes()).unwrap();
    Ok(())
}

fn output_operator(outfile: &mut File, op: &Operator)
    -> Result<(), CompilerError> {
    let op_str = match *op {
        Operator::Add => "+",
        Operator::Sub => "-",
        Operator::Mult => "*",
        Operator::MatMult => unimplemented!(),
        Operator::Div => "/",
        Operator::Mod => "%",
        Operator::Pow => unimplemented!(),
        Operator::LShift => "<<",
        Operator::RShift => ">>",
        Operator::BitOr => "|",
        Operator::BitXor => "^",
        Operator::BitAnd => "&",
        Operator::FloorDiv => unimplemented!()
    };

    outfile.write_all(op_str.as_bytes()).unwrap();
    Ok(())
}
