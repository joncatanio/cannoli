mod util;
mod errors;
mod local;

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::sync::Mutex;
use std::iter::Peekable;
use std::slice::Iter;
use std::collections::HashSet;
use clap::ArgMatches;

use super::lexer::Lexer;
use super::parser;
use super::parser::ast::*;
use self::errors::CompilerError;
use self::local::Local;

const INDENT: &str = "    ";

// Needed global mutable variables that could represent compilation state, I
// didn't want to do it this way but have no time for a refactor.
lazy_static! {
    /// Vector that manages what modules need to be compiled
    static ref MOD_QUEUE: Mutex<Vec<String>> = Mutex::new(vec![]);
    /// HashSet of all modules that have already been compiled
    static ref MOD_IMPORTS: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
    /// Path to the root directory of the src files determined by the main mod
    static ref SRC_ROOT: Mutex<String> = Mutex::new(String::new());
}

pub fn compile(file: &str, opt_args: Option<&ArgMatches>)
    -> Result<(), CompilerError> {
    let (src_root, module) = util::get_file_prefix(file)?;
    *SRC_ROOT.lock().unwrap() = src_root;

    let mut filename = "main.rs".to_string();
    filename.insert_str(0, &*SRC_ROOT.lock().unwrap());

    let result = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(filename);
    let mut outfile = if result.is_err() {
        return Err(CompilerError::IOError(format!("{:?}", result)));
    } else {
        result.unwrap()
    };

    // Write out the simple 'main.rs' file contents
    outfile.write_all(format!("extern crate cannolib;\nmod cannoli_mods;\n\n\
        fn main() {{\n{}cannoli_mods::main::execute()\n}}", INDENT)
        .as_bytes()).unwrap();

    // Output all modules to 'cannoli_mods.rs'
    let mut filename = "cannoli_mods.rs".to_string();
    filename.insert_str(0, &*SRC_ROOT.lock().unwrap());

    let result = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(filename);
    let mut outfile = if result.is_err() {
        return Err(CompilerError::IOError(format!("{:?}", result)));
    } else {
        result.unwrap()
    };

    output_file_headers(&mut outfile)?;

    let mut is_main = true;
    queue_module(&module);
    loop {
        let modules = MOD_QUEUE.lock().unwrap().clone();

        MOD_QUEUE.lock().unwrap().clear();
        if modules.is_empty() {
            break
        }

        for module in modules.iter() {
            compile_module(&mut outfile, &module, is_main, opt_args)?;
            is_main = false
        }
    }

    Ok(())
}

fn compile_module(outfile: &mut File, module: &str, is_main: bool,
    opt_args: Option<&ArgMatches>) -> Result<(), CompilerError> {
    let mut file = format!("{}.py", module);
    file.insert_str(0, &*SRC_ROOT.lock().unwrap());
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
        if is_main && args.is_present("parse") {
            println!("AST: {:?}", ast);
            return Ok(())
        }
    }

    if is_main {
        return output_main(outfile, &ast);
    } else {
        return output_module(outfile, module, &ast);
    }
}

fn queue_module(module: &str) {
    let compile = MOD_IMPORTS.lock().unwrap().get(module).is_none();

    if compile {
        MOD_QUEUE.lock().unwrap().push(module.to_string());
        MOD_IMPORTS.lock().unwrap().insert(module.to_string());
    }
}

fn output_file_headers(outfile: &mut File) -> Result<(), CompilerError> {
    outfile.write("extern crate cannolib;\n".as_bytes()).unwrap();

    Ok(())
}

fn output_module_headers(outfile: &mut File, indent: usize)
    -> Result<(), CompilerError> {
    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write_all("use cannolib;\n".as_bytes()).unwrap();
    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write_all("use std;\n".as_bytes()).unwrap();

    Ok(())
}

fn output_main(outfile: &mut File, ast: &Ast) -> Result<(), CompilerError> {
    let body = match *ast {
        Ast::Module { ref body } => body
    };

    // Setup main function and initialize scope list
    outfile.write_all("pub mod main {\n".as_bytes()).unwrap();

    // Output per-module headers
    output_module_headers(outfile, 1)?;

    outfile.write(INDENT.repeat(1).as_bytes()).unwrap();
    outfile.write_all("pub fn execute() {\n".as_bytes()).unwrap();
    outfile.write(INDENT.repeat(2).as_bytes()).unwrap();
    outfile.write_all("let mut cannoli_scope_list: \
        Vec<std::rc::Rc<std::cell::RefCell<std::collections::HashMap<String, \
        cannolib::Value>>>> = Vec::new();\n".as_bytes()).unwrap();
    outfile.write(INDENT.repeat(2).as_bytes()).unwrap();
    outfile.write_all("cannoli_scope_list.push(\
        std::rc::Rc::new(std::cell::RefCell::new(\
        cannolib::builtin::get_scope())));\n".as_bytes()).unwrap();
    outfile.write(INDENT.repeat(2).as_bytes()).unwrap();
    outfile.write_all("cannoli_scope_list.push(\
        std::rc::Rc::new(std::cell::RefCell::new(\
        std::collections::HashMap::new())));\n".as_bytes()).unwrap();
    outfile.write(INDENT.repeat(2).as_bytes()).unwrap();
    outfile.write_all("cannoli_scope_list.last_mut().unwrap().borrow_mut()\
        .insert(\"__name__\".to_string(), cannolib::Value::Str(\"__main__\"\
        .to_string()));\n".as_bytes()).unwrap();

    output_stmts(outfile, false, 2, body)?;

    outfile.write(INDENT.repeat(1).as_bytes()).unwrap();
    outfile.write_all("}\n".as_bytes()).unwrap();
    outfile.write_all("}\n".as_bytes()).unwrap();
    Ok(())
}

fn output_module(outfile: &mut File, module: &str, ast: &Ast)
    -> Result<(), CompilerError> {
    let body = match *ast {
        Ast::Module { ref body } => body
    };

    // Import module will return a Value::Object, this will be assigned to
    // the module name in the caller's scope
    outfile.write_all(format!("pub mod {} {{\n", module).as_bytes()).unwrap();

    // Output per-module headers
    output_module_headers(outfile, 1)?;

    outfile.write(INDENT.repeat(1).as_bytes()).unwrap();
    outfile.write_all("pub fn import_module() -> cannolib::Value {\n"
        .as_bytes()).unwrap();
    outfile.write(INDENT.repeat(2).as_bytes()).unwrap();
    outfile.write_all("let mut cannoli_scope_list: \
        Vec<std::rc::Rc<std::cell::RefCell<std::collections::HashMap<String, \
        cannolib::Value>>>> = Vec::new();\n".as_bytes()).unwrap();
    outfile.write(INDENT.repeat(2).as_bytes()).unwrap();
    outfile.write_all("cannoli_scope_list.push(\
        std::rc::Rc::new(std::cell::RefCell::new(\
        cannolib::builtin::get_scope())));\n".as_bytes()).unwrap();
    outfile.write(INDENT.repeat(2).as_bytes()).unwrap();
    outfile.write_all("cannoli_scope_list.push(\
        std::rc::Rc::new(std::cell::RefCell::new(\
        std::collections::HashMap::new())));\n".as_bytes()).unwrap();
    outfile.write(INDENT.repeat(2).as_bytes()).unwrap();
    outfile.write_all(format!("cannoli_scope_list.last_mut().unwrap()\
        .borrow_mut().insert(\"__name__\".to_string(), cannolib::Value::Str(\
        \"{}\".to_string()));\n", module).as_bytes()).unwrap();

    output_stmts(outfile, false, 2, body)?;

    outfile.write(INDENT.repeat(2).as_bytes()).unwrap();
    outfile.write("let cannoli_module_tbl = cannoli_scope_list.last().unwrap()\
        .borrow().clone();\n".as_bytes()).unwrap();
    outfile.write(INDENT.repeat(2).as_bytes()).unwrap();
    outfile.write_all("cannolib::Value::Object { tbl: std::rc::Rc::new(\
        std::cell::RefCell::new(cannoli_module_tbl)) }\n".as_bytes()).unwrap();
    outfile.write(INDENT.repeat(1).as_bytes()).unwrap();
    outfile.write_all("}\n".as_bytes()).unwrap();
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
        Statement::Return { .. } => output_stmt_return(outfile, indent, stmt),
        Statement::Delete { .. } => unimplemented!(),
        Statement::Assign { .. } => output_stmt_assign(outfile,
            class_scope, indent, stmt),
        Statement::AugAssign { .. } => unimplemented!(),
        Statement::AnnAssign { .. } => unimplemented!(),
        Statement::For { .. } => output_stmt_for(outfile, indent, stmt),
        Statement::While { .. } => output_stmt_while(outfile, indent, stmt),
        Statement::If { .. }    => output_stmt_if(outfile, indent, stmt),
        Statement::With { .. } => unimplemented!(),
        Statement::Raise { .. } => unimplemented!(),
        Statement::Try { .. } => unimplemented!(),
        Statement::Assert { .. } => unimplemented!(),
        Statement::Import { .. } => output_stmt_import(outfile, indent, stmt),
        Statement::ImportFrom { .. } => output_stmt_import_from(outfile,
            indent, stmt),
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
    let mut prefix = String::new();
    let local = Local::new();

    if class_scope {
        prefix.push_str("cannoli_object_tbl");
    } else {
        prefix.push_str("cannoli_scope_list.last_mut().unwrap().borrow_mut()");
    }

    // Setup function signature and append to the scope list
    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write("let move_scope = cannoli_scope_list.clone();\n"
        .as_bytes()).unwrap();
    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write(format!("let mut {} = cannolib::Value::Function(std::rc::Rc\
        ::new(move |cannoli_func_args: Vec<cannolib::Value>| \
        -> cannolib::Value {{\n", local).as_bytes()).unwrap();
    outfile.write(INDENT.repeat(indent + 1).as_bytes()).unwrap();
    outfile.write("let mut cannoli_scope_list = move_scope.clone();\n"
        .as_bytes()).unwrap();
    outfile.write(INDENT.repeat(indent + 1).as_bytes()).unwrap();
    outfile.write("cannoli_scope_list.push(std::rc::Rc::new(std::cell::RefCell\
        ::new(std::collections::HashMap::new())));\n".as_bytes()).unwrap();

    // setup parameters
    output_parameters(outfile, indent + 1, args)?;
    output_stmts(outfile, false, indent + 1, body)?;

    // output default return value (None) and closing bracket
    outfile.write(INDENT.repeat(indent + 1).as_bytes()).unwrap();
    outfile.write("cannolib::Value::None\n".as_bytes()).unwrap();
    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write("}));\n".as_bytes()).unwrap();
    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write(format!("{}.insert(\"{}\".to_string(), {});\n",
        prefix, name, local).as_bytes()).unwrap();
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

    // Add meta information into the table
    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write(format!("cannoli_object_tbl.insert(\"__class__\"\
        .to_string(), cannolib::Value::Str(\"{}\".to_string()));\n",
        name).as_bytes()).unwrap();

    // Add the new class definition to the current scope table
    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write_all(format!("cannoli_scope_list.last_mut().unwrap()\
        .borrow_mut().insert(\"{}\".to_string(), cannolib::Value::Class {{ \
        tbl: cannoli_object_tbl }});\n", name).as_bytes()).unwrap();

    Ok(())
}

fn output_stmt_return(outfile: &mut File, indent: usize, stmt: &Statement)
    -> Result<(), CompilerError> {
    let value = match *stmt {
        Statement::Return { ref value } => value,
        _ => unreachable!()
    };

    match *value {
        Some(ref value) => {
            let value_local = output_expr(outfile, indent, value)?;

            outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
            outfile.write_all(format!("return {};\n", value_local)
                .as_bytes()).unwrap();
        },
        None => {
            outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
            outfile.write_all("return cannolib::Value::None;\n"
                .as_bytes()).unwrap();
        }
    }

    Ok(())
}

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
        prefix.push_str("cannoli_scope_list.last_mut().unwrap().borrow_mut()");
    }

    // For each target determine if it's a Name/Attribute/Subscript and handle
    // each differently. Name values should be inserted into the current scope
    // list. Attributes should call a member function on Value that modifies
    // the object's internal tbl. Subscript should also call a member function
    // but only work on lists and dicts.
    let value_local = output_expr(outfile, indent, value)?;
    for target in targets.iter() {
        unpack_values(outfile, indent, &value_local, target)?;
    }
    Ok(())
}

// TODO add support for for-else
fn output_stmt_for(outfile: &mut File, indent: usize, stmt: &Statement)
    -> Result<(), CompilerError> {
    let (target, iter, body, _orelse) = match *stmt {
        Statement::For { ref target, ref iter, ref body, ref orelse } =>
            (target, iter, body, orelse),
        _ => unreachable!()
    };
    let iter_local = Local::new();
    let seq_local = output_expr(outfile, indent, iter)?;

    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write_all(format!("let mut {} = {}.clone_seq().into_iter();\n",
        iter_local, seq_local).as_bytes()).unwrap();
    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write_all("loop {\n".as_bytes()).unwrap();

    let next_local = Local::new();
    outfile.write(INDENT.repeat(indent + 1).as_bytes()).unwrap();
    outfile.write_all(format!("let mut {} = if let Some(val) = \
        {}.next() {{ val }} else {{ break }};\n", next_local,
        iter_local).as_bytes()).unwrap();

    unpack_values(outfile, indent + 1, &next_local, target)?;
    output_stmts(outfile, false, indent + 1, body)?;

    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write_all("}\n".as_bytes()).unwrap();
    Ok(())
}

fn output_stmt_while(outfile: &mut File, indent: usize, stmt: &Statement)
    -> Result<(), CompilerError> {
    let (test, body, orelse) = match *stmt {
        Statement::While { ref test, ref body, ref orelse } =>
            (test, body, orelse),
        _ => unreachable!()
    };

    let condition = output_expr(outfile, indent, test)?;
    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write_all(format!("while ({}).to_bool() {{\n",
        condition).as_bytes()).unwrap();

    output_stmts(outfile, false, indent + 1, body)?;

    // update the condition variable
    let loop_cond = output_expr(outfile, indent + 1, test)?;
    outfile.write(INDENT.repeat(indent + 1).as_bytes()).unwrap();
    outfile.write_all(format!("{} = {};\n", condition, loop_cond)
        .as_bytes()).unwrap();

    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write_all("}\n".as_bytes()).unwrap();

    if !orelse.is_empty() {
        // Negate the WHILE condition and add an if-statement
        let condition = output_expr(outfile, indent, test)?;
        outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
        outfile.write_all(format!("if !({}).to_bool() {{\n",
            condition).as_bytes()).unwrap();

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
    let test_local = output_expr(outfile, indent, test)?;
    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write_all(format!("if ({}).to_bool() {{\n", test_local)
        .as_bytes()).unwrap();

    // `then` body
    output_stmts(outfile, false, indent + 1, body)?;

    // closing decorator
    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write_all("}\n".as_bytes()).unwrap();

    // check for elif/else
    if !orelse.is_empty() {
        outfile.write_all(" else {\n".as_bytes()).unwrap();
        output_stmts(outfile, false, indent + 1, orelse)?;
        outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
        outfile.write_all("}\n".as_bytes()).unwrap();
    }
    Ok(())
}

fn output_stmt_import(outfile: &mut File, indent: usize, stmt: &Statement)
    -> Result<(), CompilerError> {
    let names = match *stmt {
        Statement::Import { ref names } => names,
        _ => unreachable!()
    };

    for name in names.iter() {
        let (name, asname) = match *name {
            Alias::Alias { ref name, ref asname } => (name, asname)
        };
        let alias = match *asname {
            Some(ref alias) => alias,
            None => name
        };

        queue_module(name);

        outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
        outfile.write(format!("use cannoli_mods::{};\n", name)
            .as_bytes()).unwrap();
        outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
        outfile.write(format!("cannoli_scope_list.last_mut().unwrap()\
            .borrow_mut().insert(\"{}\".to_string(), {}::import_module());\n",
            alias, name).as_bytes()).unwrap();
    }

    outfile.flush().unwrap();
    Ok(())
}

// TODO extend the from-import functionality to include the directory levels,
// dot imports "from . import x", etc.
fn output_stmt_import_from(outfile: &mut File, indent: usize, stmt: &Statement)
    -> Result<(), CompilerError> {
    let (module, names, _level) = match *stmt {
        Statement::ImportFrom { ref module, ref names, ref level } =>
            (module, names, level),
        _ => unreachable!()
    };

    // Check for wildcard
    let wildcard_present = if names.len() == 1 {
        match names[0] {
            Alias::Alias { ref name, .. } => {
                if name == "*" { true } else { false }
            }
        }
    } else {
        false
    };

    let mod_name = if let &Some(ref mod_name) = module {
        mod_name
    } else {
        unimplemented!()
    };

    queue_module(mod_name);

    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write(format!("use cannoli_mods::{};\n", mod_name)
        .as_bytes()).unwrap();
    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write(format!("cannoli_scope_list.last_mut().unwrap()\
        .borrow_mut().extend(cannolib::split_object({}::import_module(), ",
        mod_name).as_bytes()).unwrap();

    let mut members_arg = String::new();
    if wildcard_present {
        members_arg.push_str("None");
    } else {
        members_arg.push_str("Some(vec![");

        for name in names.iter() {
            // TODO check if '*' is used and throw and error at this point
            let (name, asname) = match *name {
                Alias::Alias { ref name, ref asname } => (name, asname)
            };
            let alias = match *asname {
                Some(ref alias) => alias,
                None => name
            };

            members_arg.push_str(&format!("(\"{}\".to_string(), \"{}\"\
                .to_string()),", name, alias));
        }
        members_arg.pop();
        members_arg.push_str("])");
    }

    outfile.write_all(format!("{}));\n", members_arg).as_bytes()).unwrap();
    Ok(())
}

fn output_stmt_expr(outfile: &mut File, indent: usize, stmt: &Statement)
    -> Result<(), CompilerError> {
    let expr = match *stmt {
        Statement::Expr { ref value } => value,
        _ => unreachable!()
    };

    output_expr(outfile, indent, expr)?;
    Ok(())
}

/// Outputs an expression always yielding a cannolib::Value. This value is
/// stored into a Local, this is done to avoid borrowing conflicts and should
/// be mitigated by the optimizer (copy propagation).
///
/// # Arguments
/// * `outfile` - the file that is being written out to
/// * `indent` - defines the indent level for definitions
/// * `expr` - Expression subtree of the AST that is being output
fn output_expr(outfile: &mut File, indent: usize, expr: &Expression)
    -> Result<Local, CompilerError> {
    match *expr {
        Expression::BoolOp { .. } => output_expr_boolop(outfile, indent, expr),
        Expression::BinOp { .. } => output_expr_binop(outfile, indent, expr),
        Expression::UnaryOp { .. } =>
            output_expr_unaryop(outfile, indent, expr),
        Expression::Lambda { .. } => unimplemented!(),
        Expression::If { .. } => output_expr_if(outfile, indent, expr),
        Expression::Dict { .. } => unimplemented!(),
        Expression::Set { .. } => unimplemented!(),
        Expression::ListComp { .. } =>
            output_expr_listcomp(outfile, indent, expr),
        Expression::SetComp { .. } => unimplemented!(),
        Expression::DictComp { .. } => unimplemented!(),
        Expression::Generator { .. } => unimplemented!(),
        Expression::None => unimplemented!(),
        Expression::Yield { .. } => unimplemented!(),
        Expression::YieldFrom { .. } => unimplemented!(),
        Expression::Compare { .. } => output_expr_cmp(outfile, indent, expr),
        Expression::Call { .. } => output_expr_call(outfile, indent, expr),
        Expression::Num { ref n }  => output_expr_num(outfile, indent, n),
        Expression::Str { ref s }  => output_expr_str(outfile, indent, s),
        Expression::NameConstant { ref value } =>
            output_expr_name_const(outfile, indent, value),
        Expression::Ellipsis => unimplemented!(),
        Expression::Attribute { .. } => output_expr_attr(outfile, indent, expr),
        Expression::Subscript { .. } =>
            output_expr_subscript(outfile, indent, expr),
        Expression::Starred { .. } => unimplemented!(),
        Expression::Name { .. } => output_expr_name(outfile, indent, expr),
        Expression::List { .. } => output_expr_list(outfile, indent, expr),
        Expression::Tuple { .. } => output_expr_tuple(outfile, indent, expr)
    }
}

fn output_expr_boolop(outfile: &mut File, indent: usize, expr: &Expression)
    -> Result<Local, CompilerError> {
    let mut output = String::new();
    let (op, values) = match *expr {
        Expression::BoolOp { ref op, ref values } => (op, values),
        _ => unreachable!()
    };
    let mut expr_iter = values.iter();
    let local = Local::new();

    // A BoolOp should always have at least two values, in order to work with
    // the Rust && and || ops the operands must be `bool`s, each expression
    // will output their bool value and the entire expression will be wrapped
    // back into a Value::Bool. There is room for optimization with this
    // especially if there is a large chain of BoolOps.
    let expr_local = output_expr(outfile, indent, expr_iter.next().unwrap())?;
    output.push_str(&INDENT.repeat(indent));
    output.push_str(&format!("let mut {} = cannolib::Value::Bool(({})\
        .to_bool()", local, expr_local));

    for expr in expr_iter {
        let expr_local = output_expr(outfile, indent, expr)?;
        output.push_str(&format!(" {} ({}).to_bool()",
            output_bool_operator(op)?, expr_local));
    }
    output.push_str(");\n");

    outfile.write_all(output.as_bytes()).unwrap();
    Ok(local)
}

fn output_expr_binop(outfile: &mut File, indent: usize, expr: &Expression)
    -> Result<Local, CompilerError> {
    let mut output = String::new();
    let (left, op, right) = match *expr {
        Expression::BinOp { ref left, ref op, ref right } => (left, op, right),
        _ => unreachable!()
    };
    let local = Local::new();
    let left_local = output_expr(outfile, indent, left)?;
    let right_local = output_expr(outfile, indent, right)?;

    output.push_str(&INDENT.repeat(indent));
    output.push_str(&format!("let mut {} = {} {} {};\n", local,
        left_local, output_operator(op)?, right_local));

    outfile.write_all(output.as_bytes()).unwrap();
    Ok(local)
}

fn output_expr_unaryop(outfile: &mut File, indent: usize, expr: &Expression)
    -> Result<Local, CompilerError> {
    let mut output = String::new();
    let (op, operand) = match *expr {
        Expression::UnaryOp { ref op, ref operand } => (op, operand),
        _ => unreachable!()
    };
    let local = Local::new();
    let operand_local = output_expr(outfile, indent, operand)?;

    output.push_str(&INDENT.repeat(indent));
    match *op {
        UnaryOperator::Invert => {
            output.push_str(&format!("let mut {} = !{};\n", local,
                operand_local));
        },
        UnaryOperator::Not => {
            output.push_str(&format!("let mut {} = ({}).logical_not();\n",
                local, operand_local));
        },
        UnaryOperator::UAdd => unimplemented!(),
        UnaryOperator::USub => {
            output.push_str(&format!("let mut {} = -{};\n", local,
                operand_local));
        }
    }

    outfile.write_all(output.as_bytes()).unwrap();
    Ok(local)
}

fn output_expr_if(outfile: &mut File, indent: usize, expr: &Expression)
    -> Result<Local, CompilerError> {
    let mut output = String::new();
    let (test, body, orelse) = match *expr {
        Expression::If { ref test, ref body, ref orelse } =>
            (test, body, orelse),
        _ => unreachable!()
    };
    let local = Local::new();
    let test_local = output_expr(outfile, indent, test)?;
    let body_local = output_expr(outfile, indent, body)?;
    let orelse_local = output_expr(outfile, indent, orelse)?;

    output.push_str(&INDENT.repeat(indent));
    output.push_str(&format!("let mut {} = if ({}).to_bool() {{ {} }} \
        else {{ {} }};\n", local, test_local, body_local, orelse_local));

    outfile.write_all(output.as_bytes()).unwrap();
    Ok(local)
}

fn output_expr_listcomp(outfile: &mut File, indent: usize, expr: &Expression)
    -> Result<Local, CompilerError> {
    let mut output = String::new();
    let (elt, generators) = match *expr {
        Expression::ListComp { ref elt, ref generators } => (elt, generators),
        _ => unreachable!()
    };
    let local = Local::new();
    let list_local = Local::new();

    // Isolate the list comprehension inorder to ensure targets don't get
    // mapped to the current scope list, then start building output list
    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write_all("cannoli_scope_list.push(std::rc::Rc::new(\
        std::cell::RefCell::new(std::collections::HashMap::new())));\n"
        .as_bytes()).unwrap();
    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write_all(&format!("let mut {} = vec![];\n", list_local)
        .as_bytes()).unwrap();

    let gen_iter = generators.iter().peekable();
    output_nested_listcomp(outfile, indent, &list_local, elt, gen_iter)?;

    output.push_str(&INDENT.repeat(indent));
    output.push_str(&format!("let mut {} = cannolib::Value::List(\
        std::rc::Rc::new(std::cell::RefCell::new(cannolib::ListType::new(\
        {}))));\n", local, list_local));
    output.push_str(&INDENT.repeat(indent));
    output.push_str("cannoli_scope_list.pop();\n");

    outfile.write_all(output.as_bytes()).unwrap();
    Ok(local)
}

// Tail recurse on nested fors in a list comprehension this was done to print
// matching brackets in a much cleaner way
fn output_nested_listcomp(outfile: &mut File, indent: usize, list_local: &Local,
    elt: &Expression, mut gen_iter: Peekable<Iter<Comprehension>>)
    -> Result<(), CompilerError> {
    let comp = match gen_iter.next() {
        Some(comp) => comp,
        None => return Ok(()) // Base case
    };
    let (target, iter, ifs) = match *comp {
        Comprehension::Comprehension { ref target, ref iter, ref ifs} =>
            (target, iter, ifs)
    };
    let iter_local = Local::new();
    let seq_local = output_expr(outfile, indent, iter)?;

    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write_all(format!("let mut {} = {}.clone_seq()\
        .into_iter();\n", iter_local, seq_local).as_bytes()).unwrap();
    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write_all("loop {\n".as_bytes()).unwrap();

    let next_local = Local::new();
    outfile.write(INDENT.repeat(indent + 1).as_bytes()).unwrap();
    outfile.write_all(format!("let mut {} = if let Some(val) = \
        {}.next() {{ val }} else {{ break }};\n", next_local,
        iter_local).as_bytes()).unwrap();
    unpack_values(outfile, indent + 1, &next_local, target)?;

    let mut conds = vec![];
    for cond in ifs.iter() {
        let cond_local = output_expr(outfile, indent + 1, cond)?;
        conds.push(cond_local);
    }

    if !conds.is_empty() {
        outfile.write(INDENT.repeat(indent + 1).as_bytes()).unwrap();
        outfile.write("if ".as_bytes()).unwrap();

        let mut cond_iter = conds.iter().peekable();
        loop {
            let cond = match cond_iter.next() {
                Some(cond) => cond,
                None => break
            };
            outfile.write(format!("({}).to_bool()", cond)
                .as_bytes()).unwrap();

            if let Some(_) = cond_iter.peek() {
                outfile.write(" && ".as_bytes()).unwrap();
            }
        }

        outfile.write_all(" {\n".as_bytes()).unwrap();
    }

    let cond_indent = if conds.is_empty() { indent + 1 } else { indent + 2 };
    // For the most nested element we want to append the 'elt'
    if let None = gen_iter.peek() {
        let elt_local = output_expr(outfile, cond_indent, elt)?;

        outfile.write(INDENT.repeat(cond_indent).as_bytes()).unwrap();
        outfile.write(format!("{}.push({});\n", list_local, elt_local)
            .as_bytes()).unwrap();
    }

    // recurse before we output closing brackets
    output_nested_listcomp(outfile, cond_indent, list_local, elt, gen_iter)?;

    if !conds.is_empty() {
        outfile.write(INDENT.repeat(indent + 1).as_bytes()).unwrap();
        outfile.write("}\n".as_bytes()).unwrap();
    }
    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write("}\n".as_bytes()).unwrap();

    Ok(())
}

fn output_expr_cmp(outfile: &mut File, indent: usize, expr: &Expression)
    -> Result<Local, CompilerError> {
    let mut output = String::new();
    let (left, ops, comparators) = match *expr {
        Expression::Compare { ref left, ref ops, ref comparators } =>
            (left, ops, comparators),
        _ => unreachable!()
    };
    let local = Local::new();
    let left_local = output_expr(outfile, indent, left)?;

    output.push_str(&INDENT.repeat(indent));
    output.push_str(&format!("let mut {} = cannolib::Value::Bool(({}", local,
        left_local));

    let mut cmp_iter = ops.iter().zip(comparators.iter()).peekable();
    loop {
        match cmp_iter.next() {
            Some((op, comparator)) => {
                let cmp_local = output_expr(outfile, indent, comparator)?;
                output.push_str(&format!("{})", output_cmp_operator(op,
                    &cmp_local)?));

                if let Some(_) = cmp_iter.peek() {
                    let cmp_local = output_expr(outfile, indent, comparator)?;
                    output.push_str(&format!(" && ({}", cmp_local));
                }
            },
            None => break
        }
    }
    output.push_str(");\n");

    outfile.write_all(output.as_bytes()).unwrap();
    Ok(local)
}

fn output_expr_call(outfile: &mut File, indent: usize, expr: &Expression)
    -> Result<Local, CompilerError> {
    let mut output = String::new();
    let (func, args, _keywords) = match *expr {
        Expression::Call { ref func, ref args, ref keywords } =>
            (func, args, keywords),
        _ => unreachable!()
    };
    let local = Local::new();

    output.push_str(&INDENT.repeat(indent));
    match **func {
        Expression::Attribute { ref value, ref attr, .. } => {
            let value_local = output_expr(outfile, indent, value)?;
            output.push_str(&format!("let mut {} = cannolib::call_member({}, \
                \"{}\", vec![", local, value_local, attr));
        },
        _ => {
            let func_local = output_expr(outfile, indent, func)?;
            output.push_str(&format!("let mut {} = {}.call(vec![",
                local, func_local));
        }
    }

    let mut args_iter = args.iter().peekable();
    loop {
        match args_iter.next() {
            Some(expr) => {
                let expr_local = output_expr(outfile, indent, expr)?;
                output.push_str(&format!("{}", expr_local));

                if let Some(_) = args_iter.peek() {
                    output.push_str(", ");
                }
            },
            None => break
        }
    }
    output.push_str("]);\n");

    outfile.write_all(output.as_bytes()).unwrap();
    Ok(local)
}

fn output_expr_num(outfile: &mut File, indent: usize, num: &Number)
    -> Result<Local, CompilerError> {
    let mut output = String::new();
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
    let local = Local::new();

    output.push_str(&INDENT.repeat(indent));
    output.push_str(&format!("let mut {} = {};\n", local, out_str));

    outfile.write_all(output.as_bytes()).unwrap();
    Ok(local)
}

fn output_expr_str(outfile: &mut File, indent: usize, string: &String)
    -> Result<Local, CompilerError> {
    let mut output = String::new();
    let out_str = format!("cannolib::Value::Str(\"{}\".to_string())", string);
    let local = Local::new();

    output.push_str(&INDENT.repeat(indent));
    output.push_str(&format!("let mut {} = {};\n", local, out_str));

    outfile.write_all(output.as_bytes()).unwrap();
    Ok(local)
}

fn output_expr_name_const(outfile: &mut File, indent: usize, value: &Singleton)
    -> Result<Local, CompilerError> {
    let mut output = String::new();
    let out_str = match *value {
        Singleton::None  => format!("cannolib::Value::None"),
        Singleton::True  => format!("cannolib::Value::Bool(true)"),
        Singleton::False => format!("cannolib::Value::Bool(false)"),
    };
    let local = Local::new();

    output.push_str(&INDENT.repeat(indent));
    output.push_str(&format!("let mut {} = {};\n", local, out_str));

    outfile.write_all(output.as_bytes()).unwrap();
    Ok(local)
}

fn output_expr_attr(outfile: &mut File, indent: usize, expr: &Expression)
    -> Result<Local, CompilerError> {
    let mut output = String::new();
    let (value, attr, _ctx) = match *expr {
        Expression::Attribute { ref value, ref attr, ref ctx } =>
            (value, attr, ctx),
        _ => unreachable!()
    };
    let local = Local::new();
    let value_local = output_expr(outfile, indent, value)?;

    output.push_str(&INDENT.repeat(indent));
    output.push_str(&format!("let mut {} = {}.get_attr(\"{}\");\n", local,
        value_local, attr));

    outfile.write_all(output.as_bytes()).unwrap();
    Ok(local)
}

fn output_expr_subscript(outfile: &mut File, indent: usize, expr: &Expression)
    -> Result<Local, CompilerError> {
    let mut output = String::new();
    let (value, slice, _ctx) = match *expr {
        Expression::Subscript { ref value, ref slice, ref ctx } =>
            (value, slice, ctx),
        _ => unreachable!()
    };
    let local = Local::new();
    let value_local = output_expr(outfile, indent, value)?;

    match **slice {
        Slice::Slice { ref lower, ref upper, ref step } => {
            let lower_arg = match *lower {
                Some(ref expr) => {
                    let expr_local = output_expr(outfile, indent, expr)?;
                    format!("Some({})", expr_local)
                },
                None => "None".to_string()
            };
            let upper_arg = match *upper {
                Some(ref expr) => {
                    let expr_local = output_expr(outfile, indent, expr)?;
                    format!("Some({})", expr_local)
                },
                None => "None".to_string()
            };
            let step_arg = match *step {
                Some(ref expr) => {
                    let expr_local = output_expr(outfile, indent, expr)?;
                    format!("Some({})", expr_local)
                },
                None => "None".to_string()
            };

            output.push_str(&INDENT.repeat(indent));
            output.push_str(&format!("let mut {} = {}.slice({}, {}, {});\n",
                local, value_local, lower_arg, upper_arg, step_arg));
        },
        Slice::ExtSlice { .. } => unimplemented!(),
        Slice::Index { ref value } => {
            let index_local = output_expr(outfile, indent, value)?;

            output.push_str(&INDENT.repeat(indent));
            output.push_str(&format!("let mut {} = {}.index({});\n", local,
                value_local, index_local));
        }
    }

    outfile.write_all(output.as_bytes()).unwrap();
    Ok(local)
}

fn output_expr_name(outfile: &mut File, indent: usize, expr: &Expression)
    -> Result<Local, CompilerError> {
    let mut output = String::new();
    let (id, _ctx) = match *expr {
        Expression::Name { ref id, ref ctx } => (id, ctx),
        _ => unreachable!()
    };
    let local = Local::new();

    output.push_str(&INDENT.repeat(indent));
    output.push_str(&format!("let mut {} = cannolib::lookup_value(\
        &cannoli_scope_list, \"{}\");\n", local, id));

    outfile.write_all(output.as_bytes()).unwrap();
    Ok(local)
}

fn output_expr_list(outfile: &mut File, indent: usize, expr: &Expression)
    -> Result<Local, CompilerError> {
    let mut output = String::new();
    let (elts, _ctx) = match *expr {
        Expression::List { ref elts, ref ctx } => (elts, ctx),
        _ => unreachable!()
    };
    let local = Local::new();

    output.push_str(&INDENT.repeat(indent));
    output.push_str(&format!("let mut cannoli_list_builder = Vec::new();\n"));

    for elt in elts.iter() {
        let elt_local = output_expr(outfile, indent, elt)?;

        output.push_str(&INDENT.repeat(indent));
        output.push_str(&format!("cannoli_list_builder.push({});\n",
            elt_local));
    }

    output.push_str(&INDENT.repeat(indent));
    output.push_str(&format!("let mut {} = cannolib::Value::List(\
        std::rc::Rc::new(std::cell::RefCell::new(cannolib::ListType::new(\
        cannoli_list_builder))));\n", local));

    outfile.write_all(output.as_bytes()).unwrap();
    Ok(local)
}

fn output_expr_tuple(outfile: &mut File, indent: usize, expr: &Expression)
    -> Result<Local, CompilerError> {
    let mut output = String::new();
    let (elts, _ctx) = match *expr {
        Expression::Tuple { ref elts, ref ctx } => (elts, ctx),
        _ => unreachable!()
    };
    let local = Local::new();

    output.push_str(&INDENT.repeat(indent));
    output.push_str(&format!("let mut cannoli_tuple_builder = Vec::new();\n"));

    for elt in elts.iter() {
        let elt_local = output_expr(outfile, indent, elt)?;

        output.push_str(&INDENT.repeat(indent));
        output.push_str(&format!("cannoli_tuple_builder.push({});\n",
            elt_local));
    }

    output.push_str(&INDENT.repeat(indent));
    output.push_str(&format!("let mut {} = cannolib::Value::Tuple(\
        cannolib::TupleType::new(cannoli_tuple_builder));\n", local));

    outfile.write_all(output.as_bytes()).unwrap();
    Ok(local)
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
            .borrow_mut().insert(\"{}\".to_string(), cannoli_func_args_iter\
            .next().expect(\"expected {} positional args\"));\n", arg_name,
            args.len()).as_bytes()).unwrap();
    }

    outfile.flush().unwrap();
    Ok(())
}

fn output_bool_operator(op: &BoolOperator)
    -> Result<String, CompilerError> {
    let op_str = match *op {
        BoolOperator::And => "&&",
        BoolOperator::Or  => "||",
    };
    Ok(op_str.to_string())
}

fn output_operator(op: &Operator)
    -> Result<String, CompilerError> {
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
    Ok(op_str.to_string())
}

// TODO I'll have to do something interesting for is/in, maybe append a
// function call to the LHS Value and wrap the RHS in parens.
fn output_cmp_operator(op: &CmpOperator, val: &Local)
    -> Result<String, CompilerError> {
    let op_str = match *op {
        CmpOperator::EQ => format!(" == {}", val),
        CmpOperator::NE => format!(" != {}", val),
        CmpOperator::LT => format!(" < {}", val),
        CmpOperator::LE => format!(" <= {}", val),
        CmpOperator::GT => format!(" > {}", val),
        CmpOperator::GE => format!(" >= {}", val),
        CmpOperator::Is => unimplemented!(),
        CmpOperator::IsNot => unimplemented!(),
        CmpOperator::In => format!(".contained_in(&{})", val),
        CmpOperator::NotIn => format!(".not_contained_in(&{})", val)
    };
    Ok(op_str)
}

// TODO implement recursive logic for target unpacking, currently this only
// supports single-level unpacking consider something like
// ex: for a, ((b, c), d) in [(1, ((2, 3), 4))]: ...
fn unpack_values(outfile: &mut File, indent: usize, packed_values: &Local,
    target: &Expression) -> Result<(), CompilerError> {
    match *target {
        Expression::Name { ref id, .. } => {
            outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
            outfile.write_all(format!("cannoli_scope_list.last_mut().unwrap()\
                .borrow_mut().insert(\"{}\".to_string(), {});\n", id,
                packed_values).as_bytes()).unwrap();
        },
        Expression::Attribute { ref value, ref attr, .. } => {
            let base_local = output_expr(outfile, indent, value)?;
            outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
            outfile.write_all(format!("cannolib::attr_assign({}, \"{}\", {}\
                );\n", base_local, attr, packed_values).as_bytes()).unwrap();
        },
        Expression::List { .. } => {
            unimplemented!()
        },
        Expression::Tuple { ref elts, .. } => {
            for (ndx, elt) in elts.iter().enumerate() {
                match *elt {
                    Expression::Name { ref id, .. } => {
                        outfile.write(INDENT.repeat(indent).as_bytes())
                            .unwrap();
                        outfile.write_all(format!("cannoli_scope_list.\
                            last_mut().unwrap().borrow_mut().insert(\"{}\"\
                            .to_string(), {}.index(cannolib::Value::Number(\
                            cannolib::NumericType::Integer({}))));\n", id,
                            packed_values, ndx).as_bytes()).unwrap();
                    },
                    _ => unimplemented!()
                }
            }
        },
        _ => panic!("unable to unpack values")
    }
    Ok(())
}
