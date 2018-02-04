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
        Statement::While { .. } => output_stmt_while(outfile, indent, stmt),
        Statement::If { .. }    => output_stmt_if(outfile, indent, stmt),
        Statement::Expr { .. }  => output_stmt_expr(outfile, indent, stmt),
        _ => unimplemented!()
    }
}

fn output_stmt_while(outfile: &mut File, indent: usize, stmt: &Statement)
    -> Result<(), CompilerError> {
    let (test, body, orelse) = match *stmt {
        Statement::While { ref test, ref body, ref orelse } =>
            (test, body, orelse),
        _ => unreachable!()
    };

    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write_all("while (".as_bytes()).unwrap();
    output_expr(outfile, test)?;
    outfile.write_all(").to_bool() {\n".as_bytes()).unwrap();

    output_stmts(outfile, indent + 1, body)?;

    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write_all("}\n".as_bytes()).unwrap();

    if !orelse.is_empty() {
        // Negate the WHILE condition and add an if-statement
        outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
        outfile.write_all("if !(".as_bytes()).unwrap();
        output_expr(outfile, test)?;
        outfile.write_all(").to_bool() {\n".as_bytes()).unwrap();

        output_stmts(outfile, indent + 1, orelse)?;

        outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
        outfile.write_all("}\n".as_bytes()).unwrap();
    }
    Ok(())
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
    outfile.write_all("}".as_bytes()).unwrap();

    // check for elif/else
    if !orelse.is_empty() {
        if let Some(&&Statement::If { .. }) = orelse.iter().peekable().peek() {
            outfile.write_all(" else".as_bytes()).unwrap();
            output_stmts(outfile, indent, orelse)?;
        } else {
            outfile.write_all(" else {\n".as_bytes()).unwrap();
            output_stmts(outfile, indent + 1, orelse)?;
            outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
            outfile.write_all("}\n".as_bytes()).unwrap();
        };
    }
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
        Expression::BoolOp { .. }  => output_expr_boolop(outfile, expr),
        Expression::BinOp { .. }   => output_expr_binop(outfile, expr),
        Expression::UnaryOp { .. } => output_expr_unaryop(outfile, expr),
        Expression::If { .. }      => output_expr_if(outfile, expr),
        Expression::Compare { .. } => output_expr_cmp(outfile, expr),
        Expression::Num { ref n }  => output_expr_num(outfile, n),
        Expression::Str { ref s }  => output_expr_str(outfile, s),
        Expression::NameConstant { ref value } =>
            output_expr_name_const(outfile, value),
        _ => unimplemented!()
    }
}

fn output_expr_boolop(outfile: &mut File, expr: &Expression)
    -> Result<(), CompilerError> {
    let (op, values) = match *expr {
        Expression::BoolOp { ref op, ref values } => (op, values),
        _ => unreachable!()
    };
    let mut expr_iter = values.iter();

    // A BoolOp should always have at least two values, in order to work with
    // the Rust && and || ops the operands must be `bool`s, each expression
    // will output their bool value and the entire expression will be wrapped
    // back into a Value::Bool. There is room for optimization with this
    // especially if there is a large chain of BoolOps.
    outfile.write_all("cannolib::Value::Bool((".as_bytes()).unwrap();
    output_expr(outfile, expr_iter.next().unwrap())?;
    outfile.write_all(").to_bool()".as_bytes()).unwrap();
    for expr in expr_iter {
        outfile.write_all(" ".as_bytes()).unwrap();
        output_bool_operator(outfile, op)?;
        outfile.write_all(" (".as_bytes()).unwrap();
        output_expr(outfile, expr)?;
        outfile.write_all(").to_bool()".as_bytes()).unwrap();
    }
    outfile.write_all(")".as_bytes()).unwrap();

    Ok(())
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

fn output_expr_unaryop(outfile: &mut File, expr: &Expression)
    -> Result<(), CompilerError> {
    let (op, operand) = match *expr {
        Expression::UnaryOp { ref op, ref operand } => (op, operand),
        _ => unreachable!()
    };

    match *op {
        UnaryOperator::Invert => {
            outfile.write_all("!".as_bytes()).unwrap();
            output_expr(outfile, operand)?;
        },
        UnaryOperator::Not => {
            outfile.write_all("(".as_bytes()).unwrap();
            output_expr(outfile, operand)?;
            outfile.write_all(").logical_not()".as_bytes()).unwrap();
        },
        UnaryOperator::UAdd => unimplemented!(),
        UnaryOperator::USub => {
            outfile.write_all("-".as_bytes()).unwrap();
            output_expr(outfile, operand)?;
        }
    }
    Ok(())

}

fn output_expr_if(outfile: &mut File, expr: &Expression)
    -> Result<(), CompilerError> {
    let (test, body, orelse) = match *expr {
        Expression::If { ref test, ref body, ref orelse } =>
            (test, body, orelse),
        _ => unreachable!()
    };

    outfile.write_all("if (".as_bytes()).unwrap();
    output_expr(outfile, test)?;
    outfile.write_all(").to_bool() { ".as_bytes()).unwrap();
    output_expr(outfile, body)?;
    outfile.write_all(" } else { ".as_bytes()).unwrap();
    output_expr(outfile, orelse)?;
    outfile.write_all(" }".as_bytes()).unwrap();
    Ok(())
}

fn output_expr_cmp(outfile: &mut File, expr: &Expression)
    -> Result<(), CompilerError> {
    let (left, ops, comparators) = match *expr {
        Expression::Compare { ref left, ref ops, ref comparators } =>
            (left, ops, comparators),
        _ => unreachable!()
    };

    outfile.write_all("cannolib::Value::Bool((".as_bytes()).unwrap();
    output_expr(outfile, left)?;

    let mut cmp_iter = ops.iter().zip(comparators.iter()).peekable();
    loop {
        match cmp_iter.next() {
            Some((op, comparator)) => {
                outfile.write_all(" ".as_bytes()).unwrap();
                output_cmp_operator(outfile, op)?;
                outfile.write_all(" ".as_bytes()).unwrap();
                output_expr(outfile, comparator)?;
                outfile.write_all(")".as_bytes()).unwrap();

                if let Some(_) = cmp_iter.peek() {
                    outfile.write_all(" && (".as_bytes()).unwrap();
                    output_expr(outfile, comparator)?;
                }
            }
            None => break
        }
    }

    outfile.write_all(")".as_bytes()).unwrap();
    Ok(())
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

fn output_expr_str(outfile: &mut File, string: &String)
    -> Result<(), CompilerError> {
    let out_str = format!("cannolib::Value::Str(\"{}\".to_string())", string);

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

fn output_bool_operator(outfile: &mut File, op: &BoolOperator)
    -> Result<(), CompilerError> {
    let op_str = match *op {
        BoolOperator::And => "&&",
        BoolOperator::Or  => "||",
    };

    outfile.write_all(op_str.as_bytes()).unwrap();
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

// TODO I'll have to do something interesting for is/in, maybe append a
// function call to the LHS Value and wrap the RHS in parens.
fn output_cmp_operator(outfile: &mut File, op: &CmpOperator)
    -> Result<(), CompilerError> {
    let op_str = match *op {
        CmpOperator::EQ => "==",
        CmpOperator::NE => "!=",
        CmpOperator::LT => "<",
        CmpOperator::LE => "<=",
        CmpOperator::GT => ">",
        CmpOperator::GE => ">=",
        CmpOperator::Is => unimplemented!(),
        CmpOperator::IsNot => unimplemented!(),
        CmpOperator::In => unimplemented!(),
        CmpOperator::NotIn => unimplemented!()
    };

    outfile.write_all(op_str.as_bytes()).unwrap();
    Ok(())
}
