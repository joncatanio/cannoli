mod util;
mod errors;
mod local;

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::sync::Mutex;
use std::iter::Peekable;
use std::slice::Iter;
use std::collections::{HashMap, HashSet};
use clap::ArgMatches;
use cannolib;

use super::lexer::Lexer;
use super::parser;
use super::parser::ast::*;
use self::errors::CompilerError;
use self::local::Local;
use self::util::TrackedScope;

const INDENT: &str = "    ";

type ClassScope = Option<HashMap<String, (usize, Option<String>)>>;

// Needed global mutable variables that could represent compilation state, I
// didn't want to do it this way but have no time for a refactor.
lazy_static! {
    /// Vector that manages what modules need to be compiled
    static ref MOD_QUEUE: Mutex<Vec<String>> = Mutex::new(vec![]);
    /// HashSet of all modules that have already been compiled
    static ref MOD_IMPORTS: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
    /// Path to the root directory of the src files determined by the main mod
    static ref SRC_ROOT: Mutex<String> = Mutex::new(String::new());
    /// All of the built-in modules provided by Python
    static ref BUILTIN_MODS: HashSet<&'static str> = init_modules();
}

fn init_modules() -> HashSet<&'static str> {
    let mut modules = HashSet::new();
    modules.insert("sys");
    modules.insert("math");
    modules
}

/// Starts compilation of a module
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

    // Gather the current scope elements and create the scope Vec that will be
    // used in this compiler to output a more efficient scoping system.
    let mut scope = TrackedScope::new();
    scope.push_scope(cannolib::builtin::get_mapping());
    let mut current_scope = util::gather_scope(body, 0, false)?;

    // Insert meta data into the scope on the compiler side, this is also
    // setup below to be used at runtime. Then appened the updated scope.
    let offset = current_scope.len();
    current_scope.insert("__name__".to_string(), (offset, None));

    let capacity = current_scope.len();
    scope.push_scope(current_scope);

    outfile.write(INDENT.repeat(1).as_bytes()).unwrap();
    outfile.write_all("pub fn execute() {\n".as_bytes()).unwrap();
    outfile.write(INDENT.repeat(2).as_bytes()).unwrap();
    outfile.write_all("let mut cannoli_scope_list: \
        Vec<std::rc::Rc<std::cell::RefCell<Vec<cannolib::Value>>>> = \
        Vec::new();\n".as_bytes()).unwrap();
    outfile.write(INDENT.repeat(2).as_bytes()).unwrap();
    outfile.write_all("cannoli_scope_list.push(\
        std::rc::Rc::new(std::cell::RefCell::new(\
        cannolib::builtin::get_scope())));\n".as_bytes()).unwrap();

    // Append a fresh scope_list with a defined capacity
    outfile.write(INDENT.repeat(2).as_bytes()).unwrap();
    outfile.write_all(format!("let mut scope_list_setup: Vec<cannolib::Value> \
        = Vec::with_capacity({});\n", capacity).as_bytes()).unwrap();
    outfile.write(INDENT.repeat(2).as_bytes()).unwrap();
    outfile.write_all(format!("scope_list_setup.resize({}, \
        cannolib::Value::Undefined);\n", capacity).as_bytes()).unwrap();
    outfile.write(INDENT.repeat(2).as_bytes()).unwrap();
    outfile.write_all("cannoli_scope_list.push(\
        std::rc::Rc::new(std::cell::RefCell::new(scope_list_setup)));\n"
        .as_bytes()).unwrap();

    // Output metadata from the map
    let (ndx, offset, _) = scope.lookup_value("__name__")?;
    outfile.write(INDENT.repeat(2).as_bytes()).unwrap();
    outfile.write_all(format!("cannoli_scope_list[{}].borrow_mut()[{}] = \
        cannolib::Value::Str(\"__main__\".to_string());\n", ndx, offset)
        .as_bytes()).unwrap();

    println!("SCOPE: {:?}", scope);
    output_stmts(outfile, &mut scope, None, 2, body)?;
    println!("SCOPE: {:?}", scope);

    outfile.write(INDENT.repeat(1).as_bytes()).unwrap();
    outfile.write_all("}\n".as_bytes()).unwrap();
    outfile.write_all("}\n".as_bytes()).unwrap();

    // Pop the main scope and the built-ins, this is to just stay consistent
    scope.pop_scope();
    scope.pop_scope();
    Ok(())
}

// TODO going to need to do something clever if we want to support modules with
// the new vector scope optimization. Since 'import_module' returns an object
// of the most recent scope table. When objects are optimized it might be a
// good time to revisit modules to see if it can be done.
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

    // TODO remove or modify, this is a dummy var for 'output_stmts' below.
    let mut scope = TrackedScope::new();

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
    outfile.write(INDENT.repeat(2).as_bytes()).unwrap();
    outfile.write_all("cannoli_scope_list.last_mut().unwrap().borrow_mut()\
        .insert(\"__module__\".to_string(), cannolib::Value::Bool(true));\n"
        .as_bytes()).unwrap();

    output_stmts(outfile, &mut scope, None, 2, body)?;

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

fn output_stmts(outfile: &mut File, scope: &mut TrackedScope,
    class_scope: ClassScope, indent: usize, stmts: &Vec<Statement>)
    -> Result<(), CompilerError> {
    for stmt in stmts.iter() {
        output_stmt(outfile, scope, class_scope.clone(), indent, stmt)?;
    }
    Ok(())
}

fn output_stmt(outfile: &mut File, scope: &mut TrackedScope,
    class_scope: ClassScope, indent: usize, stmt: &Statement)
    -> Result<(), CompilerError> {
    match *stmt {
        Statement::FunctionDef { .. } => output_stmt_funcdef(outfile,
            scope, class_scope, indent, stmt),
        Statement::ClassDef { .. } =>
            output_stmt_classdef(outfile, scope, indent, stmt),
        Statement::Return { .. } =>
            output_stmt_return(outfile, scope, indent, stmt),
        Statement::Delete { .. } => unimplemented!(),
        Statement::Assign { .. } => output_stmt_assign(outfile, scope,
            class_scope, indent, stmt),
        Statement::AugAssign { .. } => output_stmt_aug_assign(outfile,
            scope, class_scope, indent, stmt),
        Statement::AnnAssign { .. } => output_stmt_ann_assign(outfile,
            scope, class_scope, indent, stmt),
        Statement::For { .. } =>
            output_stmt_for(outfile, scope, indent, stmt),
        Statement::While { .. } =>
            output_stmt_while(outfile, scope, indent, stmt),
        Statement::If { .. } =>
            output_stmt_if(outfile, scope, indent, stmt),
        Statement::With { .. } => unimplemented!(),
        Statement::Raise { .. } => unimplemented!(),
        Statement::Try { .. } => unimplemented!(),
        Statement::Assert { .. } => unimplemented!(),
        Statement::Import { .. } =>
            output_stmt_import(outfile, scope, indent, stmt),
        Statement::ImportFrom { .. } =>
            output_stmt_import_from(outfile, scope, indent, stmt),
        Statement::Global { .. } => unimplemented!(),
        Statement::Nonlocal { .. } => unimplemented!(),
        Statement::Expr { .. }  =>
            output_stmt_expr(outfile, scope, indent, stmt),
        Statement::Pass => unimplemented!(),
        Statement::Break => unimplemented!(),
        Statement::Continue => unimplemented!()
    }
}

fn output_stmt_funcdef(outfile: &mut File, scope: &mut TrackedScope,
    class_scope: ClassScope, indent: usize, stmt: &Statement)
    -> Result<(), CompilerError> {
    let (name, args, body, _decorator_list, _returns) = match *stmt {
        Statement::FunctionDef { ref name, ref args, ref body,
            ref decorator_list, ref returns } =>
            (name, args, body, decorator_list, returns),
        _ => unreachable!()
    };
    let local = Local::new();
    let param_map = util::gather_func_params(args, 0)?;
    let mut current_scope = util::gather_scope(body, param_map.len(), false)?;
    current_scope.extend(param_map);

    let capacity = current_scope.len();
    scope.push_scope(current_scope);

    // Setup function signature and append to the scope list
    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write("let move_scope = cannoli_scope_list.clone();\n"
        .as_bytes()).unwrap();
    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write(format!("let mut {} = cannolib::Value::Function(std::rc::Rc\
        ::new(move |cannoli_func_args: Vec<cannolib::Value>, mut kwargs: \
        std::collections::HashMap<String, cannolib::Value>| -> cannolib::Value \
        {{\n", local).as_bytes()).unwrap();
    outfile.write(INDENT.repeat(indent + 1).as_bytes()).unwrap();
    outfile.write("let mut cannoli_scope_list = move_scope.clone();\n"
        .as_bytes()).unwrap();

    // Append a fresh scope_list with a defined capacity
    outfile.write(INDENT.repeat(indent + 1).as_bytes()).unwrap();
    outfile.write_all(format!("let mut scope_list_setup: Vec<cannolib::Value> \
        = Vec::with_capacity({});\n", capacity).as_bytes()).unwrap();
    outfile.write(INDENT.repeat(indent + 1).as_bytes()).unwrap();
    outfile.write_all(format!("scope_list_setup.resize({}, \
        cannolib::Value::Undefined);\n", capacity).as_bytes()).unwrap();
    outfile.write(INDENT.repeat(indent + 1).as_bytes()).unwrap();
    outfile.write_all("cannoli_scope_list.push(\
        std::rc::Rc::new(std::cell::RefCell::new(scope_list_setup)));\n"
        .as_bytes()).unwrap();

    // setup parameters
    output_parameters(outfile, scope, indent + 1, args)?;

    // TODO not outputting kwargs right now since it isn't needed for the
    // ray tracer and seems to be pretty complex. It might be something we axe.
    //outfile.write(INDENT.repeat(indent + 1).as_bytes()).unwrap();
    //outfile.write("cannoli_scope_list.last_mut().unwrap().borrow_mut()\
    //    .extend(kwargs);\n".as_bytes()).unwrap();

    output_stmts(outfile, scope, None, indent + 1, body)?;

    // output default return value (None) and closing bracket
    outfile.write(INDENT.repeat(indent + 1).as_bytes()).unwrap();
    outfile.write("cannolib::Value::None\n".as_bytes()).unwrap();
    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write("}));\n".as_bytes()).unwrap();

    // assign the function pointer to either the class scope or default scope
    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    if let Some(tbl) = class_scope {
        let ndx = if let Some(&(ref ndx, _)) = tbl.get(name) {
            ndx
        } else {
            return Err(CompilerError::NameError(name.to_string()))
        };

        outfile.write(format!("cannoli_object_vec[{}] = {}; // func: '{}'\n",
            ndx, local, name).as_bytes()).unwrap();
    } else {
        let (ndx, offset, _) = scope.lookup_value(name)?;
        outfile.write(format!("cannoli_scope_list[{}].borrow_mut()[{}] = {}; \
            // func: '{}'\n", ndx, offset, local, name).as_bytes()).unwrap();
    }

    scope.pop_scope();
    outfile.flush().unwrap();
    Ok(())
}

fn output_stmt_classdef(outfile: &mut File, scope: &mut TrackedScope,
    indent: usize, stmt: &Statement) -> Result<(), CompilerError> {
    let (name, _, _, body, _) = match *stmt {
        Statement::ClassDef { ref name, ref bases, ref keywords, ref body,
            ref decorator_list } => (name, bases, keywords, body,
            decorator_list),
        _ => unreachable!()
    };
    let mut class_tbl = util::gather_scope(body, 0, true)?;
    let offset = class_tbl.len();
    class_tbl.insert("__name__".to_string(), (offset, None));

    // Add the class to the scope class map
    scope.insert_class(name, class_tbl.clone());

    // Setup the HashMap that maps the offsets in the object vector
    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write("let mut cannoli_object_tbl = \
        std::collections::HashMap::new();\n".as_bytes()).unwrap();

    for (member, &(ref ndx, _)) in class_tbl.iter() {
        outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
        outfile.write(format!("cannoli_object_tbl.insert(\"{}\".to_string(), \
            {});\n", member, ndx).as_bytes()).unwrap();
    }

    // Initialize the object vector
    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write(format!("let mut cannoli_object_vec = Vec::\
        with_capacity({});\n", class_tbl.len()).as_bytes()).unwrap();
    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write_all(format!("cannoli_object_vec.resize({}, cannolib::Value::\
        Undefined);\n", class_tbl.len()).as_bytes()).unwrap();

    // Add meta information into the table
    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write(format!("cannoli_object_vec[{}] = cannolib::Value::Str(\
        \"{}\".to_string());\n", class_tbl.get("__name__").unwrap().0, name)
        .as_bytes()).unwrap();

    output_stmts(outfile, scope, Some(class_tbl), indent, body)?;

    // Add the new class definition to the current scope table
    let (ndx, offset, _) = scope.lookup_value(name)?;
    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write_all(format!("cannoli_scope_list[{}].borrow_mut()[{}] = \
        cannolib::Value::Class {{ tbl: std::rc::Rc::new(cannoli_object_tbl), \
        members: cannoli_object_vec }}; // class: '{}'\n", ndx, offset, name)
        .as_bytes()).unwrap();

    Ok(())
}

fn output_stmt_return(outfile: &mut File, scope: &mut TrackedScope,
    indent: usize, stmt: &Statement) -> Result<(), CompilerError> {
    let value = match *stmt {
        Statement::Return { ref value } => value,
        _ => unreachable!()
    };

    match *value {
        Some(ref value) => {
            let value_local = output_expr(outfile, scope, indent,
                value)?;

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

fn output_stmt_assign(outfile: &mut File, scope: &mut TrackedScope,
    class_scope: ClassScope, indent: usize, stmt: &Statement)
    -> Result<(), CompilerError> {
    let (targets, value) = match *stmt {
        Statement::Assign { ref targets, ref value } => (targets, value),
        _ => unreachable!()
    };
    let value_local = output_expr(outfile, scope, indent, value)?;

    for target in targets.iter() {
        unpack_values(outfile, scope, indent, class_scope.clone(),
            &value_local, target, None)?;
    }
    Ok(())
}

// TODO ignores class_scope for now, I don't know if it even works in Python.
fn output_stmt_aug_assign(outfile: &mut File, scope: &mut TrackedScope,
    _class_scope: ClassScope, indent: usize, stmt: &Statement)
    -> Result<(), CompilerError> {
    let (target, op, value) = match *stmt {
        Statement::AugAssign { ref target, ref op, ref value } =>
            (target, op, value),
        _ => unreachable!()
    };
    let value_local = output_expr(outfile, scope, indent, value)?;

    match *target {
        Expression::Name { ref id, .. } => {
            let local = Local::new();
            let (ndx, offset, _) = scope.lookup_value(id)?;

            outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
            outfile.write_all(format!("let mut {} = cannoli_scope_list[{}]\
                .borrow()[{}].clone();\n", local, ndx, offset)
                .as_bytes()).unwrap();
            outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
            outfile.write_all(format!("cannoli_scope_list[{}].borrow_mut()[{}] \
                = {};\n", ndx, offset, output_operator(&local, op,
                &value_local)?).as_bytes()).unwrap();
        },
        _ => panic!("illegal expression for augmented assignment")
    }
    Ok(())
}

fn output_stmt_ann_assign(outfile: &mut File, scope: &mut TrackedScope,
    class_scope: ClassScope, indent: usize, stmt: &Statement)
    -> Result<(), CompilerError> {
    let (target, annotation, value) = match *stmt {
        Statement::AnnAssign { ref target, ref annotation, ref value } => {
            let value = match *value {
                Some(ref value) => value,
                None => return Ok(())
            };
            (target, annotation, value)
        },
        _ => unreachable!()
    };
    let value_local = output_expr(outfile, scope, indent, value)?;

    unpack_values(outfile, scope, indent, class_scope.clone(),
        &value_local, target, Some(annotation))?;

    Ok(())
}

// TODO add support for for-else
fn output_stmt_for(outfile: &mut File, scope: &mut TrackedScope, indent: usize,
    stmt: &Statement) -> Result<(), CompilerError> {
    let (target, iter, body, _orelse) = match *stmt {
        Statement::For { ref target, ref iter, ref body, ref orelse } =>
            (target, iter, body, orelse),
        _ => unreachable!()
    };
    let iter_local = Local::new();
    let seq_local = output_expr(outfile, scope, indent, iter)?;

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

    unpack_values(outfile, scope, indent + 1, None, &next_local,
        target, None)?;
    output_stmts(outfile, scope, None, indent + 1, body)?;

    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write_all("}\n".as_bytes()).unwrap();
    Ok(())
}

fn output_stmt_while(outfile: &mut File, scope: &mut TrackedScope,
    indent: usize, stmt: &Statement) -> Result<(), CompilerError> {
    let (test, body, orelse) = match *stmt {
        Statement::While { ref test, ref body, ref orelse } =>
            (test, body, orelse),
        _ => unreachable!()
    };

    let condition = output_expr(outfile, scope, indent, test)?;
    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write_all(format!("while ({}).to_bool() {{\n",
        condition).as_bytes()).unwrap();

    output_stmts(outfile, scope, None, indent + 1, body)?;

    // update the condition variable
    let loop_cond = output_expr(outfile, scope, indent + 1, test)?;
    outfile.write(INDENT.repeat(indent + 1).as_bytes()).unwrap();
    outfile.write_all(format!("{} = {};\n", condition, loop_cond)
        .as_bytes()).unwrap();

    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write_all("}\n".as_bytes()).unwrap();

    if !orelse.is_empty() {
        // Negate the WHILE condition and add an if-statement
        let condition = output_expr(outfile, scope, indent, test)?;
        outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
        outfile.write_all(format!("if !({}).to_bool() {{\n",
            condition).as_bytes()).unwrap();

        output_stmts(outfile, scope, None, indent + 1, orelse)?;

        outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
        outfile.write_all("}\n".as_bytes()).unwrap();
    }
    Ok(())
}

fn output_stmt_if(outfile: &mut File, scope: &mut TrackedScope, indent: usize,
    stmt: &Statement) -> Result<(), CompilerError> {
    let (test, body, orelse) = match *stmt {
        Statement::If { ref test, ref body, ref orelse } =>
            (test, body, orelse),
        _ => unreachable!()
    };

    // guard and decorators
    let test_local = output_expr(outfile, scope, indent, test)?;
    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write_all(format!("if ({}).to_bool() {{\n", test_local)
        .as_bytes()).unwrap();

    // `then` body
    output_stmts(outfile, scope, None, indent + 1, body)?;

    // closing decorator
    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write_all("}".as_bytes()).unwrap();

    // check for elif/else
    if !orelse.is_empty() {
        outfile.write_all(" else {\n".as_bytes()).unwrap();
        output_stmts(outfile, scope, None, indent + 1, orelse)?;
        outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
        outfile.write_all("}\n".as_bytes()).unwrap();
    } else {
        outfile.write_all("\n".as_bytes()).unwrap();
    }

    Ok(())
}

fn output_stmt_import(outfile: &mut File, scope: &mut TrackedScope, indent: usize,
    stmt: &Statement) -> Result<(), CompilerError> {
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
        let (ndx, offset, _) = scope.lookup_value(alias)?;

        if BUILTIN_MODS.contains(&name[..]) {
            outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
            outfile.write(format!("cannoli_scope_list[{}].borrow_mut()[{}] = \
                cannolib::builtin::{}::import_module(); // mod alias: '{}'\n",
                ndx, offset, name, alias).as_bytes()).unwrap();
            return Ok(())
        }

        queue_module(name);

        outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
        outfile.write(format!("use cannoli_mods::{};\n", name)
            .as_bytes()).unwrap();
        outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
        outfile.write(format!("cannoli_scope_list[{}].borrow_mut()[{}] = \
            {}::import_module(); // mod alias: '{}'\n", ndx, offset, name,
            alias).as_bytes()).unwrap();
    }

    outfile.flush().unwrap();
    Ok(())
}

// TODO extend the from-import functionality to include the directory levels,
// dot imports "from . import x", etc.
// TODO support vector scope optimization, we need to figure out a way to
// handle wildcards.
fn output_stmt_import_from(outfile: &mut File, _scope: &mut TrackedScope,
    indent: usize, stmt: &Statement) -> Result<(), CompilerError> {
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

fn output_stmt_expr(outfile: &mut File, scope: &mut TrackedScope, indent: usize,
    stmt: &Statement) -> Result<(), CompilerError> {
    let expr = match *stmt {
        Statement::Expr { ref value } => value,
        _ => unreachable!()
    };

    output_expr(outfile, scope, indent, expr)?;
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
fn output_expr(outfile: &mut File, scope: &mut TrackedScope, indent: usize,
    expr: &Expression) -> Result<Local, CompilerError> {
    match *expr {
        Expression::BoolOp { .. } =>
            output_expr_boolop(outfile, scope, indent, expr),
        Expression::BinOp { .. } =>
            output_expr_binop(outfile, scope, indent, expr),
        Expression::UnaryOp { .. } =>
            output_expr_unaryop(outfile, scope, indent, expr),
        Expression::Lambda { .. } => unimplemented!(),
        Expression::If { .. } =>
            output_expr_if(outfile, scope, indent, expr),
        Expression::Dict { .. } => unimplemented!(),
        Expression::Set { .. } => unimplemented!(),
        Expression::ListComp { .. } =>
            output_expr_listcomp(outfile, scope, indent, expr),
        Expression::SetComp { .. } => unimplemented!(),
        Expression::DictComp { .. } => unimplemented!(),
        Expression::Generator { .. } => unimplemented!(),
        Expression::None => unimplemented!(),
        Expression::Yield { .. } => unimplemented!(),
        Expression::YieldFrom { .. } => unimplemented!(),
        Expression::Compare { .. } =>
            output_expr_cmp(outfile, scope, indent, expr),
        Expression::Call { .. } =>
            output_expr_call(outfile, scope, indent, expr),
        Expression::Num { ref n }  =>
            output_expr_num(outfile, scope, indent, n),
        Expression::Str { ref s }  =>
            output_expr_str(outfile, scope, indent, s),
        Expression::NameConstant { ref value } =>
            output_expr_name_const(outfile, scope, indent, value),
        Expression::Ellipsis => unimplemented!(),
        Expression::Attribute { .. } =>
            output_expr_attr(outfile, scope, indent, expr),
        Expression::Subscript { .. } =>
            output_expr_subscript(outfile, scope, indent, expr),
        Expression::Starred { .. } => unimplemented!(),
        Expression::Name { .. } =>
            output_expr_name(outfile, scope, indent, expr),
        Expression::List { .. } =>
            output_expr_list(outfile, scope, indent, expr),
        Expression::Tuple { .. } =>
            output_expr_tuple(outfile, scope, indent, expr)
    }
}

fn output_expr_boolop(outfile: &mut File, scope: &mut TrackedScope, indent: usize,
    expr: &Expression) -> Result<Local, CompilerError> {
    let (op, values) = match *expr {
        Expression::BoolOp { ref op, ref values } => (op, values),
        _ => unreachable!()
    };
    let mut expr_iter = values.iter();
    let local = Local::new();
    let expr_local = output_expr(outfile, scope, indent,
        expr_iter.next().unwrap())?;

    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    match *op {
        BoolOperator::And => outfile.write_all(format!("let mut {} = \
            if {}.to_bool() {{\n", local, expr_local).as_bytes()).unwrap(),
        BoolOperator::Or  => outfile.write_all(format!("let mut {} = \
            if !{}.to_bool() {{\n", local, expr_local).as_bytes()).unwrap(),
    }

    rec_output_bool_op(outfile, scope, indent + 1, op, &expr_local,
        expr_iter)?;

    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    match *op {
        BoolOperator::And => outfile.write_all("} else { \
            cannolib::Value::Bool(false) };\n".as_bytes()).unwrap(),
        BoolOperator::Or  => outfile.write_all("} else { \
            cannolib::Value::Bool(true) };\n".as_bytes()).unwrap()
    }

    Ok(local)
}

/// Recursively outputs if-expressions that mirror short-circuiting
/// functionality, this has to be done due to how expressions are being written
/// out. It's certainly not ideal but must be done.
fn rec_output_bool_op(outfile: &mut File, scope: &mut TrackedScope, indent: usize,
    op: &BoolOperator, last: &Local, mut iter: Iter<Expression>)
    -> Result<(), CompilerError> {
    let expr = match iter.next() {
        Some(expr) => expr,
        None => {
            outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
            outfile.write(format!("cannolib::Value::Bool({}.to_bool())\n", last)
                .as_bytes()).unwrap();
            return Ok(())
        }
    };
    let expr_local = output_expr(outfile, scope, indent, expr)?;

    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    match *op {
        BoolOperator::And => outfile.write_all(format!("if {}.to_bool() {{\n",
            expr_local).as_bytes()).unwrap(),
        BoolOperator::Or  => outfile.write_all(format!("if !{}.to_bool() {{\n",
            expr_local).as_bytes()).unwrap(),
    }

    rec_output_bool_op(outfile, scope, indent + 1, op, &expr_local,
        iter)?;

    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    match *op {
        BoolOperator::And => outfile.write_all("} else { \
            cannolib::Value::Bool(false) }\n".as_bytes()).unwrap(),
        BoolOperator::Or  => outfile.write_all("} else { \
            cannolib::Value::Bool(true) }\n".as_bytes()).unwrap()
    }
    Ok(())
}

fn output_expr_binop(outfile: &mut File, scope: &mut TrackedScope, indent: usize,
    expr: &Expression) -> Result<Local, CompilerError> {
    let mut output = String::new();
    let (left, op, right) = match *expr {
        Expression::BinOp { ref left, ref op, ref right } => (left, op, right),
        _ => unreachable!()
    };
    let local = Local::new();
    let left_local = output_expr(outfile, scope, indent, left)?;
    let right_local = output_expr(outfile, scope, indent, right)?;

    output.push_str(&INDENT.repeat(indent));
    output.push_str(&format!("let mut {} = {};\n", local,
        output_operator(&left_local, op, &right_local)?));

    outfile.write_all(output.as_bytes()).unwrap();
    Ok(local)
}

fn output_expr_unaryop(outfile: &mut File, scope: &mut TrackedScope, indent: usize,
    expr: &Expression) -> Result<Local, CompilerError> {
    let mut output = String::new();
    let (op, operand) = match *expr {
        Expression::UnaryOp { ref op, ref operand } => (op, operand),
        _ => unreachable!()
    };
    let local = Local::new();
    let operand_local = output_expr(outfile, scope, indent, operand)?;

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

fn output_expr_if(outfile: &mut File, scope: &mut TrackedScope, indent: usize,
    expr: &Expression) -> Result<Local, CompilerError> {
    let mut output = String::new();
    let (test, body, orelse) = match *expr {
        Expression::If { ref test, ref body, ref orelse } =>
            (test, body, orelse),
        _ => unreachable!()
    };
    let local = Local::new();
    let test_local = output_expr(outfile, scope, indent, test)?;
    let body_local = output_expr(outfile, scope, indent, body)?;
    let orelse_local = output_expr(outfile, scope, indent, orelse)?;

    output.push_str(&INDENT.repeat(indent));
    output.push_str(&format!("let mut {} = if ({}).to_bool() {{ {} }} \
        else {{ {} }};\n", local, test_local, body_local, orelse_local));

    outfile.write_all(output.as_bytes()).unwrap();
    Ok(local)
}

fn output_expr_listcomp(outfile: &mut File, scope: &mut TrackedScope,
    indent: usize, expr: &Expression) -> Result<Local, CompilerError> {
    let mut output = String::new();
    let (elt, generators) = match *expr {
        Expression::ListComp { ref elt, ref generators } => (elt, generators),
        _ => unreachable!()
    };
    let local = Local::new();
    let list_local = Local::new();
    let current_scope = util::gather_comp_targets(generators, 0)?;
    let capacity = current_scope.len();

    scope.push_scope(current_scope);

    // Isolate the list comprehension inorder to ensure targets don't get
    // mapped to the current scope list, then start building output list
    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write_all(format!("let mut scope_list_setup: Vec<cannolib::Value> \
        = Vec::with_capacity({});\n", capacity).as_bytes()).unwrap();
    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write_all(format!("scope_list_setup.resize({}, \
        cannolib::Value::Undefined);\n", capacity).as_bytes()).unwrap();
    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write_all("cannoli_scope_list.push(\
        std::rc::Rc::new(std::cell::RefCell::new(scope_list_setup)));\n"
        .as_bytes()).unwrap();

    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write_all(&format!("let mut {} = vec![];\n", list_local)
        .as_bytes()).unwrap();

    let gen_iter = generators.iter().peekable();
    output_nested_listcomp(outfile, scope, indent, &list_local, elt,
        gen_iter)?;

    output.push_str(&INDENT.repeat(indent));
    output.push_str(&format!("let mut {} = cannolib::Value::List(\
        std::rc::Rc::new(std::cell::RefCell::new(cannolib::ListType::new(\
        {}))));\n", local, list_local));
    output.push_str(&INDENT.repeat(indent));
    output.push_str("cannoli_scope_list.pop();\n");

    scope.pop_scope();

    outfile.write_all(output.as_bytes()).unwrap();
    Ok(local)
}

// Tail recurse on nested fors in a list comprehension this was done to print
// matching brackets in a much cleaner way
fn output_nested_listcomp(outfile: &mut File, scope: &mut TrackedScope,
    indent: usize, list_local: &Local, elt: &Expression,
    mut gen_iter: Peekable<Iter<Comprehension>>) -> Result<(), CompilerError> {
    let comp = match gen_iter.next() {
        Some(comp) => comp,
        None => return Ok(()) // Base case
    };
    let (target, iter, ifs) = match *comp {
        Comprehension::Comprehension { ref target, ref iter, ref ifs} =>
            (target, iter, ifs)
    };
    let iter_local = Local::new();
    let seq_local = output_expr(outfile, scope, indent, iter)?;

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
    unpack_values(outfile, scope, indent + 1, None, &next_local,
        target, None)?;

    let mut conds = vec![];
    for cond in ifs.iter() {
        let cond_local = output_expr(outfile, scope, indent + 1, cond)?;
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
        let elt_local = output_expr(outfile, scope, cond_indent, elt)?;

        outfile.write(INDENT.repeat(cond_indent).as_bytes()).unwrap();
        outfile.write(format!("{}.push({});\n", list_local, elt_local)
            .as_bytes()).unwrap();
    }

    // recurse before we output closing brackets
    output_nested_listcomp(outfile, scope, cond_indent, list_local, elt,
        gen_iter)?;

    if !conds.is_empty() {
        outfile.write(INDENT.repeat(indent + 1).as_bytes()).unwrap();
        outfile.write("}\n".as_bytes()).unwrap();
    }
    outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
    outfile.write("}\n".as_bytes()).unwrap();

    Ok(())
}

fn output_expr_cmp(outfile: &mut File, scope: &mut TrackedScope, indent: usize,
    expr: &Expression) -> Result<Local, CompilerError> {
    let mut output = String::new();
    let (left, ops, comparators) = match *expr {
        Expression::Compare { ref left, ref ops, ref comparators } =>
            (left, ops, comparators),
        _ => unreachable!()
    };
    let local = Local::new();
    let left_local = output_expr(outfile, scope, indent, left)?;

    output.push_str(&INDENT.repeat(indent));
    output.push_str(&format!("let mut {} = cannolib::Value::Bool(({}", local,
        left_local));

    let mut cmp_iter = ops.iter().zip(comparators.iter()).peekable();
    loop {
        match cmp_iter.next() {
            Some((op, comparator)) => {
                let cmp_local = output_expr(outfile, scope, indent,
                    comparator)?;
                output.push_str(&format!("{})", output_cmp_operator(op,
                    &cmp_local)?));

                if let Some(_) = cmp_iter.peek() {
                    let cmp_local = output_expr(outfile, scope, indent,
                        comparator)?;
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

fn output_expr_call(outfile: &mut File, scope: &mut TrackedScope, indent: usize,
    expr: &Expression) -> Result<Local, CompilerError> {
    let mut output = String::new();
    let (func, args, keywords) = match *expr {
        Expression::Call { ref func, ref args, ref keywords } =>
            (func, args, keywords),
        _ => unreachable!()
    };
    let local = Local::new();

    output.push_str(&INDENT.repeat(indent));
    output.push_str("let mut kwargs = std::collections::HashMap::new();\n");
    for keyword in keywords.iter() {
        let (arg, value) = match *keyword {
            Keyword::Keyword { ref arg, ref value } => (arg.clone(), value)
        };
        let kw_local = output_expr(outfile, scope, indent, value)?;

        output.push_str(&INDENT.repeat(indent));
        output.push_str(&format!("kwargs.insert(\"{}\".to_string(), {});\n",
            arg.unwrap(), kw_local));
    }

    output.push_str(&INDENT.repeat(indent));
    match **func {
        Expression::Attribute { ref value, ref attr, .. } => {
            let value_local = output_expr(outfile, scope, indent,
                value)?;
            output.push_str(&format!("let mut {} = cannolib::call_member({}, \
                \"{}\", vec![", local, value_local, attr));
        },
        _ => {
            let func_local = output_expr(outfile, scope, indent, func)?;
            output.push_str(&format!("let mut {} = {}.call(vec![",
                local, func_local));
        }
    }

    let mut args_iter = args.iter().peekable();
    loop {
        match args_iter.next() {
            Some(expr) => {
                let expr_local = output_expr(outfile, scope, indent,
                    expr)?;
                output.push_str(&format!("{}", expr_local));

                if let Some(_) = args_iter.peek() {
                    output.push_str(", ");
                }
            },
            None => break
        }
    }
    output.push_str("], kwargs);\n");

    outfile.write_all(output.as_bytes()).unwrap();
    Ok(local)
}

fn output_expr_num(outfile: &mut File, _scope: &mut TrackedScope, indent: usize,
    num: &Number) -> Result<Local, CompilerError> {
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

fn output_expr_str(outfile: &mut File, _scope: &mut TrackedScope, indent: usize,
    string: &String) -> Result<Local, CompilerError> {
    let mut output = String::new();
    let out_str = format!("cannolib::Value::Str(\"{}\".to_string())", string);
    let local = Local::new();

    output.push_str(&INDENT.repeat(indent));
    output.push_str(&format!("let mut {} = {};\n", local, out_str));

    outfile.write_all(output.as_bytes()).unwrap();
    Ok(local)
}

fn output_expr_name_const(outfile: &mut File, _scope: &mut TrackedScope,
    indent: usize, value: &Singleton) -> Result<Local, CompilerError> {
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

fn output_expr_attr(outfile: &mut File, scope: &mut TrackedScope, indent: usize,
    expr: &Expression) -> Result<Local, CompilerError> {
    let mut output = String::new();
    let (value, attr, _ctx) = match *expr {
        Expression::Attribute { ref value, ref attr, ref ctx } =>
            (value, attr, ctx),
        _ => unreachable!()
    };
    let local = Local::new();
    let value_local = output_expr(outfile, scope, indent, value)?;

    output.push_str(&INDENT.repeat(indent));
    output.push_str(&format!("let mut {} = {}.get_attr(\"{}\");\n", local,
        value_local, attr));

    outfile.write_all(output.as_bytes()).unwrap();
    Ok(local)
}

fn output_expr_subscript(outfile: &mut File, scope: &mut TrackedScope,
    indent: usize, expr: &Expression) -> Result<Local, CompilerError> {
    let mut output = String::new();
    let (value, slice, _ctx) = match *expr {
        Expression::Subscript { ref value, ref slice, ref ctx } =>
            (value, slice, ctx),
        _ => unreachable!()
    };
    let local = Local::new();
    let value_local = output_expr(outfile, scope, indent, value)?;

    match **slice {
        Slice::Slice { ref lower, ref upper, ref step } => {
            let lower_arg = match *lower {
                Some(ref expr) => {
                    let expr_local = output_expr(outfile, scope,
                        indent, expr)?;
                    format!("Some({})", expr_local)
                },
                None => "None".to_string()
            };
            let upper_arg = match *upper {
                Some(ref expr) => {
                    let expr_local = output_expr(outfile, scope,
                        indent, expr)?;
                    format!("Some({})", expr_local)
                },
                None => "None".to_string()
            };
            let step_arg = match *step {
                Some(ref expr) => {
                    let expr_local = output_expr(outfile, scope,
                        indent, expr)?;
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
            let index_local = output_expr(outfile, scope,
                indent, value)?;

            output.push_str(&INDENT.repeat(indent));
            output.push_str(&format!("let mut {} = {}.index({});\n", local,
                value_local, index_local));
        }
    }

    outfile.write_all(output.as_bytes()).unwrap();
    Ok(local)
}

fn output_expr_name(outfile: &mut File, scope: &mut TrackedScope, indent: usize,
    expr: &Expression) -> Result<Local, CompilerError> {
    let mut output = String::new();
    let (id, _ctx) = match *expr {
        Expression::Name { ref id, ref ctx } => (id, ctx),
        _ => unreachable!()
    };
    let local = Local::new();
    let (ndx, offset, _) = scope.lookup_value(id)?;

    output.push_str(&INDENT.repeat(indent));
    output.push_str(&format!("let mut {} = cannoli_scope_list[{}].borrow()[{}]\
        .clone();\n", local, ndx, offset));

    outfile.write_all(output.as_bytes()).unwrap();
    Ok(local)
}

fn output_expr_list(outfile: &mut File, scope: &mut TrackedScope, indent: usize,
    expr: &Expression) -> Result<Local, CompilerError> {
    let mut output = String::new();
    let (elts, _ctx) = match *expr {
        Expression::List { ref elts, ref ctx } => (elts, ctx),
        _ => unreachable!()
    };
    let local = Local::new();

    output.push_str(&INDENT.repeat(indent));
    output.push_str(&format!("let mut cannoli_list_builder = Vec::new();\n"));

    for elt in elts.iter() {
        let elt_local = output_expr(outfile, scope, indent, elt)?;

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

fn output_expr_tuple(outfile: &mut File, scope: &mut TrackedScope, indent: usize,
    expr: &Expression) -> Result<Local, CompilerError> {
    let mut output = String::new();
    let (elts, _ctx) = match *expr {
        Expression::Tuple { ref elts, ref ctx } => (elts, ctx),
        _ => unreachable!()
    };
    let local = Local::new();

    output.push_str(&INDENT.repeat(indent));
    output.push_str(&format!("let mut cannoli_tuple_builder = Vec::new();\n"));

    for elt in elts.iter() {
        let elt_local = output_expr(outfile, scope, indent, elt)?;

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

fn output_parameters(outfile: &mut File, scope: &mut TrackedScope, indent: usize,
    params: &Arguments) -> Result<(), CompilerError> {
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
        let (ndx, offset, _) = scope.lookup_value(arg_name)?;

        outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
        outfile.write(format!("cannoli_scope_list[{}].borrow_mut()[{}] = \
            cannoli_func_args_iter.next().unwrap_or(cannolib::Value::None); \
            // param_name: '{}'\n", ndx, offset, arg_name).as_bytes()).unwrap();
    }

    outfile.flush().unwrap();
    Ok(())
}

fn output_operator(lft: &Local, op: &Operator, rht: &Local)
    -> Result<String, CompilerError> {
    let op_str = match *op {
        Operator::Add => format!("{} + {}", lft, rht),
        Operator::Sub => format!("{} - {}", lft, rht),
        Operator::Mult => format!("{} * {}", lft, rht),
        Operator::MatMult => unimplemented!(),
        Operator::Div => format!("{} / {}", lft, rht),
        Operator::Mod => format!("{} % {}", lft, rht),
        Operator::Pow => format!("{}.pow(&{})", lft, rht),
        Operator::LShift => format!("{} << {}", lft, rht),
        Operator::RShift => format!("{} >> {}", lft, rht),
        Operator::BitOr => format!("{} | {}", lft, rht),
        Operator::BitXor => format!("{} ^ {}", lft, rht),
        Operator::BitAnd => format!("{} & {}", lft, rht),
        Operator::FloorDiv => unimplemented!()
    };
    Ok(op_str)
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

// TODO add list support when needed, should be just like Tuples
/// Tail-recursive function that recursively unpacks values. Passing 'None' for
/// class_scope will default to looking up the value in the current scope,
/// this value is used when dealing with classes.
fn unpack_values(outfile: &mut File, scope: &mut TrackedScope, indent: usize,
    class_scope: ClassScope, packed_values: &Local, target: &Expression,
    annotation: Option<&Expression>) -> Result<(), CompilerError> {
    match *target {
        Expression::Name { ref id, .. } => {
            outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();

            if let Some(tbl) = class_scope {
                let ndx = if let Some(&(ref ndx, _)) = tbl.get(id) {
                    ndx
                } else {
                    return Err(CompilerError::NameError(id.to_string()))
                };

                outfile.write_all(format!("cannoli_object_vec[{}] = {}; \
                    // id: '{}'\n", ndx, packed_values, id)
                    .as_bytes()).unwrap();
            } else {
                let (ndx, offset, _) = if let Some(ann) = annotation {
                    scope.annotate(id, ann)?
                } else {
                    scope.lookup_value(id)?
                };

                outfile.write_all(format!("cannoli_scope_list[{}].borrow\
                    _mut()[{}] = {}; // id: '{}'\n", ndx, offset,
                    packed_values, id).as_bytes()).unwrap();
            }
        },
        Expression::Attribute { ref value, ref attr, .. } => {
            let base_local = output_expr(outfile, scope,
                indent, value)?;

            outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
            // This looks up the expression to see if there is has been a
            // type associated with it, if so it will try to optimize the
            // attribute assignment by accessing the object vec directly.
            if let Some((_, _, Some(vtype))) = scope.lookup_expr(value) {
                let ndx = scope.lookup_attr(&vtype, attr)?;

                outfile.write_all(format!("match {} {{\n", base_local)
                    .as_bytes()).unwrap();
                outfile.write(INDENT.repeat(indent + 1).as_bytes()).unwrap();
                outfile.write_all(format!("cannolib::Value::Object {{ members, \
                    .. }} => members.borrow_mut()[{}] = {},\n", ndx,
                    packed_values).as_bytes()).unwrap();
                outfile.write(INDENT.repeat(indent + 1).as_bytes()).unwrap();
                outfile.write_all("_ => panic!(\"bad annotation\")\n"
                    .as_bytes()).unwrap();
                outfile.write(INDENT.repeat(indent).as_bytes()).unwrap();
                outfile.write_all("}\n".as_bytes()).unwrap();
            } else {
                outfile.write_all(format!("cannolib::attr_assign({}, \"{}\", \
                    {});\n", base_local, attr, packed_values).as_bytes())
                    .unwrap();
            }
        },
        Expression::List { .. } => {
            unimplemented!()
        },
        Expression::Tuple { ref elts, .. } => {
            for (ndx, elt) in elts.iter().enumerate() {
                match *elt {
                    Expression::Name { .. }
                    | Expression::Attribute { .. }
                    | Expression::List { .. }
                    | Expression::Tuple { .. } => {
                        let local = Local::new();

                        outfile.write(INDENT.repeat(indent).as_bytes())
                            .unwrap();
                        outfile.write_all(format!("let mut {} = {}.index(\
                            cannolib::Value::Number(cannolib::NumericType::\
                            Integer({})));\n", local, packed_values, ndx)
                            .as_bytes()).unwrap();
                        unpack_values(outfile, scope, indent,
                            class_scope.clone(), &local, elt, annotation)?;
                    },
                    _ => unimplemented!()
                }
            }
        },
        _ => panic!("unable to unpack values")
    }
    Ok(())
}
