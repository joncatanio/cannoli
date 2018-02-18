mod util;
mod errors;
mod local;

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use clap::ArgMatches;

use super::lexer::Lexer;
use super::parser;
use super::parser::ast::*;
use self::errors::CompilerError;
use self::local::Local;

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
    outfile.write("extern crate cannolib;\n".as_bytes()).unwrap();

    Ok(())
}

fn output_main(outfile: &mut File, ast: &Ast) -> Result<(), CompilerError> {
    let body = match *ast {
        Ast::Module { ref body } => body
    };

    // Setup main function and initialize scope list
    outfile.write_all("fn main() {\n".as_bytes()).unwrap();
    outfile.write(INDENT.as_bytes()).unwrap();
    outfile.write_all("let mut cannoli_scope_list: \
        Vec<std::collections::HashMap<String, cannolib::Value>> = \
        Vec::new();\n".as_bytes()).unwrap();
    outfile.write(INDENT.as_bytes()).unwrap();
    outfile.write_all("cannoli_scope_list.push(\
        cannolib::builtin::get_scope());\n".as_bytes()).unwrap();
    outfile.write(INDENT.as_bytes()).unwrap();
    outfile.write_all("cannoli_scope_list.push(\
        std::collections::HashMap::new());\n".as_bytes()).unwrap();

    output_stmts(outfile, false, 1, body)?;

    outfile.write_all("}\n".as_bytes()).unwrap();
    Ok(())
}

fn output_stmts(outfile: &mut File, class_scope: bool, indent: usize,
    stmts: &Vec<Statement>) -> Result<(), CompilerError> {
    for stmt in stmts.iter() {
        output_stmt(outfile, class_scope, indent, stmt)?;
    }
    Ok(())
}

fn output_stmt(outfile: &mut File, class_scope: bool, indent: usize,
    stmt: &Statement) -> Result<(), CompilerError> {
    match *stmt {
        Statement::FunctionDef { .. } => output_stmt_funcdef(outfile,
            class_scope, indent, stmt),
        Statement::ClassDef { .. } => output_stmt_classdef(outfile,
            indent, stmt),
        Statement::Return { .. } => unimplemented!(),
        Statement::Delete { .. } => unimplemented!(),
        Statement::Assign { .. } => output_stmt_assign(outfile,
            class_scope, indent, stmt),
        Statement::AugAssign { .. } => unimplemented!(),
        Statement::AnnAssign { .. } => unimplemented!(),
        Statement::For { .. } => unimplemented!(),
        Statement::While { .. } => output_stmt_while(outfile, indent, stmt),
        Statement::If { .. }    => output_stmt_if(outfile, indent, stmt),
        Statement::With { .. } => unimplemented!(),
        Statement::Raise { .. } => unimplemented!(),
        Statement::Try { .. } => unimplemented!(),
        Statement::Assert { .. } => unimplemented!(),
        Statement::Import { .. } => unimplemented!(),
        Statement::ImportFrom { .. } => unimplemented!(),
        Statement::Global { .. } => unimplemented!(),
        Statement::Nonlocal { .. } => unimplemented!(),
        Statement::Expr { .. }  => output_stmt_expr(outfile, indent, stmt),
        Statement::Pass => unimplemented!(),
        Statement::Break => unimplemented!(),
        Statement::Continue => unimplemented!()
    }
}

fn output_stmt_funcdef(outfile: &mut File, class_scope: bool, indent: usize,
    stmt: &Statement) -> Result<(), CompilerError> {
    let (name, args, body, _decorator_list, _returns) = match *stmt {
        Statement::FunctionDef { ref name, ref args, ref body,
            ref decorator_list, ref returns } =>
            (name, args, body, decorator_list, returns),
        _ => unreachable!()
    };
    //let mangled_name = util::mangle_name(name);
    let mut prefix = String::new();

    if class_scope {
        prefix.push_str("cannoli_object_tbl");
    } else {
        prefix.push_str("cannoli_scope_list.last_mut().unwrap()");
    }

    // Setup function signature and append to the scope list
    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write(format!("{}.insert(\"{}\".to_string(), \
        cannolib::Value::Function {{ f: \
        |mut cannoli_scope_list: Vec<std::collections::HashMap<String, \
        cannolib::Value>>, cannoli_func_args: Vec<cannolib::Value>| \
        -> cannolib::Value {{\n", prefix, name).as_bytes()).unwrap();
    outfile.write(INDENT.repeat(indent + 1).as_bytes()).unwrap();
    outfile.write("cannoli_scope_list.push(std::collections::HashMap::new());\n"
        .as_bytes()).unwrap();

    // setup parameters
    output_parameters(outfile, indent + 1, args)?;
    output_stmts(outfile, false, indent + 1, body)?;

    // output default return value (None) and closing bracket
    outfile.write(INDENT.repeat(indent + 1).as_bytes()).unwrap();
    outfile.write("cannolib::Value::None\n".as_bytes()).unwrap();
    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write("}});\n".as_bytes()).unwrap();
    outfile.flush().unwrap();

    Ok(())
}

fn output_stmt_classdef(outfile: &mut File, indent: usize, stmt: &Statement)
    -> Result<(), CompilerError> {
    let (name, _, _, body, _) = match *stmt {
        Statement::ClassDef { ref name, ref bases, ref keywords, ref body,
            ref decorator_list } => (name, bases, keywords, body,
            decorator_list),
        _ => unreachable!()
    };

    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write("let mut cannoli_object_tbl = \
        std::collections::HashMap::new();\n".as_bytes()).unwrap();

    output_stmts(outfile, true, indent, body)?;

    // Add the new class definition to the current scope table
    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write_all(format!("cannoli_scope_list.last_mut().unwrap().insert(\
        \"{}\".to_string(), cannolib::Value::Object {{ class_name: \"{}\"\
        .to_string(), tbl: cannoli_object_tbl }});\n", name, name)
        .as_bytes()).unwrap();

    Ok(())
}

// TODO this will need to be reworked when Objects are implemented, I won't
// be able to simply redefine the value in Rust, I will have to modify a struct.
fn output_stmt_assign(outfile: &mut File, class_scope: bool, indent: usize,
    stmt: &Statement) -> Result<(), CompilerError> {
    let (targets, value) = match *stmt {
        Statement::Assign { ref targets, ref value } => (targets, value),
        _ => unreachable!()
    };
    let mut prefix = String::new();

    if class_scope {
        prefix.push_str("cannoli_object_tbl");
    } else {
        prefix.push_str("cannoli_scope_list.last_mut().unwrap()");
    }

    // For each target determine if it's a Name/Attribute/Subscript and handle
    // each differently. Name values should be inserted into the current scope
    // list. Attributes should call a member function on Value that modifies
    // the object's internal tbl. Subscript should also call a member function
    // but only work on lists and dicts.
    for target in targets.iter() {
        outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
        match *target {
            Expression::Name { .. } => {
                outfile.write_all(format!("{}.insert(\"", prefix)
                    .as_bytes()).unwrap();
                output_expr(outfile, false, target)?;
                outfile.write_all("\".to_string(), ".as_bytes()).unwrap();
                output_expr(outfile, true, value)?;
                outfile.write_all(");\n".as_bytes()).unwrap();
            },
            _ => panic!("unsupported assignment")
        }
    }
    Ok(())
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
    output_expr(outfile, true, test)?;
    outfile.write_all(").to_bool() {\n".as_bytes()).unwrap();

    output_stmts(outfile, false, indent + 1, body)?;

    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write_all("}\n".as_bytes()).unwrap();

    if !orelse.is_empty() {
        // Negate the WHILE condition and add an if-statement
        outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
        outfile.write_all("if !(".as_bytes()).unwrap();
        output_expr(outfile, true, test)?;
        outfile.write_all(").to_bool() {\n".as_bytes()).unwrap();

        output_stmts(outfile, false, indent + 1, orelse)?;

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
    output_expr(outfile, true, test)?;
    outfile.write_all(").to_bool() {\n".as_bytes()).unwrap();

    // `then` body
    output_stmts(outfile, false, indent + 1, body)?;

    // closing decorator
    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write_all("}".as_bytes()).unwrap();

    // check for elif/else
    if !orelse.is_empty() {
        if let Some(&&Statement::If { .. }) = orelse.iter().peekable().peek() {
            outfile.write_all(" else".as_bytes()).unwrap();
            output_stmts(outfile, false, indent, orelse)?;
        } else {
            outfile.write_all(" else {\n".as_bytes()).unwrap();
            output_stmts(outfile, false, indent + 1, orelse)?;
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
    output_expr(outfile, true, expr)?;
    outfile.write_all(";\n".as_bytes()).unwrap();
    Ok(())
}

/// Outputs an expression always yielding a Value
/// # Arguments
/// * `outfile` - the file that is being written out to
/// * `lookup` - determines whether or not an Expression::Name should be
///              looked up in the scope list
/// * `expr` - Expression subtree of the AST that is being output
fn output_expr(outfile: &mut File, lookup: bool, expr: &Expression)
    -> Result<(), CompilerError> {
    match *expr {
        Expression::BoolOp { .. } => output_expr_boolop(outfile, expr),
        Expression::BinOp { .. } => output_expr_binop(outfile, expr),
        Expression::UnaryOp { .. } => output_expr_unaryop(outfile, expr),
        Expression::Lambda { .. } => unimplemented!(),
        Expression::If { .. } => output_expr_if(outfile, expr),
        Expression::Dict { .. } => unimplemented!(),
        Expression::Set { .. } => unimplemented!(),
        Expression::ListComp { .. } => unimplemented!(),
        Expression::SetComp { .. } => unimplemented!(),
        Expression::DictComp { .. } => unimplemented!(),
        Expression::Generator { .. } => unimplemented!(),
        Expression::None => unimplemented!(),
        Expression::Yield { .. } => unimplemented!(),
        Expression::YieldFrom { .. } => unimplemented!(),
        Expression::Compare { .. } => output_expr_cmp(outfile, expr),
        Expression::Call { .. } => output_expr_call(outfile, expr),
        Expression::Num { ref n }  => output_expr_num(outfile, n),
        Expression::Str { ref s }  => output_expr_str(outfile, s),
        Expression::NameConstant { ref value } =>
            output_expr_name_const(outfile, value),
        Expression::Ellipsis => unimplemented!(),
        Expression::Attribute { .. } => unimplemented!(),
        Expression::Subscript { .. } => unimplemented!(),
        Expression::Starred { .. } => unimplemented!(),
        Expression::Name { .. } => output_expr_name(outfile, lookup, expr),
        Expression::List { .. } => unimplemented!(),
        Expression::Tuple { .. } => unimplemented!()
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
    output_expr(outfile, true, expr_iter.next().unwrap())?;
    outfile.write_all(").to_bool()".as_bytes()).unwrap();
    for expr in expr_iter {
        outfile.write_all(" ".as_bytes()).unwrap();
        output_bool_operator(outfile, op)?;
        outfile.write_all(" (".as_bytes()).unwrap();
        output_expr(outfile, true, expr)?;
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

    output_expr(outfile, true, left)?;
    outfile.write_all(" ".as_bytes()).unwrap();
    output_operator(outfile, op)?;
    outfile.write_all(" ".as_bytes()).unwrap();
    output_expr(outfile, true, right)?;
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
            output_expr(outfile, true, operand)?;
        },
        UnaryOperator::Not => {
            outfile.write_all("(".as_bytes()).unwrap();
            output_expr(outfile, true, operand)?;
            outfile.write_all(").logical_not()".as_bytes()).unwrap();
        },
        UnaryOperator::UAdd => unimplemented!(),
        UnaryOperator::USub => {
            outfile.write_all("-".as_bytes()).unwrap();
            output_expr(outfile, true, operand)?;
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
    output_expr(outfile, true, test)?;
    outfile.write_all(").to_bool() { ".as_bytes()).unwrap();
    output_expr(outfile, true, body)?;
    outfile.write_all(" } else { ".as_bytes()).unwrap();
    output_expr(outfile, true, orelse)?;
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
    output_expr(outfile, true, left)?;

    let mut cmp_iter = ops.iter().zip(comparators.iter()).peekable();
    loop {
        match cmp_iter.next() {
            Some((op, comparator)) => {
                outfile.write_all(" ".as_bytes()).unwrap();
                output_cmp_operator(outfile, op)?;
                outfile.write_all(" ".as_bytes()).unwrap();
                output_expr(outfile, true, comparator)?;
                outfile.write_all(")".as_bytes()).unwrap();

                if let Some(_) = cmp_iter.peek() {
                    outfile.write_all(" && (".as_bytes()).unwrap();
                    output_expr(outfile, true, comparator)?;
                }
            },
            None => break
        }
    }

    outfile.write_all(")".as_bytes()).unwrap();
    Ok(())
}

fn output_expr_call(outfile: &mut File, expr: &Expression)
    -> Result<(), CompilerError> {
    let (func, args, _keywords) = match *expr {
        Expression::Call { ref func, ref args, ref keywords } =>
            (func, args, keywords),
        _ => unreachable!()
    };

    match **func {
        Expression::Attribute { ref value, ref attr, .. } => {
            output_expr(outfile, true, value)?;
            outfile.write(format!(".call_member(\"{}\", \
                cannoli_scope_list.clone(), vec![", attr).as_bytes()).unwrap();
        },
        _ => {
            output_expr(outfile, true, func)?;
            outfile.write(".call(cannoli_scope_list.clone(), vec!["
                .as_bytes()).unwrap();
        }
    }

    let mut args_iter = args.iter().peekable();
    loop {
        match args_iter.next() {
            Some(expr) => {
                output_expr(outfile, true, expr)?;
                if let Some(_) = args_iter.peek() {
                    outfile.write(", ".as_bytes()).unwrap();
                }
            },
            None => break
        }
    }

    outfile.write_all("])".as_bytes()).unwrap();
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

fn output_expr_name(outfile: &mut File, lookup: bool, expr: &Expression)
    -> Result<(), CompilerError> {
    let (id, _ctx) = match *expr {
        Expression::Name { ref id, ref ctx } => (id, ctx),
        _ => unreachable!()
    };
    //let mangled_name = util::mangle_name(&id);

    if lookup {
        outfile.write_all(format!("cannolib::lookup_value(cannoli_scope_list\
            .clone(), \"{}\").clone()", id).as_bytes()).unwrap();
    } else {
        outfile.write_all(id.as_bytes()).unwrap();
    }
    Ok(())
}

fn output_parameters(outfile: &mut File, indent: usize, params: &Arguments)
    -> Result<(), CompilerError> {
    let (args, _vararg, _kwonlyargs, _kw_defaults, _kwarg, _defaults) =
    match *params {
        Arguments::Arguments { ref args, ref vararg, ref kwonlyargs,
            ref kw_defaults, ref kwarg, ref defaults } => (args, vararg,
            kwonlyargs, kw_defaults, kwarg, defaults)
    };

    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write("let mut cannoli_func_args_iter = \
        cannoli_func_args.into_iter();\n".as_bytes()).unwrap();
    for arg in args.iter() {
        let (arg_name, _arg_annotation) = match *arg {
            Arg::Arg { ref arg, ref annotation } => (arg, annotation)
        };
        //let mangled_name = util::mangle_name(&arg_name);

        outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
        outfile.write(format!("cannoli_scope_list.last_mut().unwrap()\
            .insert(\"{}\".to_string(), cannoli_func_args_iter.next()\
            .expect(\"expected {} positional args\"));\n", arg_name,
            args.len()).as_bytes()).unwrap();
    }

    outfile.flush().unwrap();
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
