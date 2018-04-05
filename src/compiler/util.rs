use regex::Regex;
use std::collections::{HashMap, HashSet};

use ::parser::ast::*;
use super::errors::CompilerError;

/// Returns the root directory of the given file and the file name sans ext
pub fn get_file_prefix(file: &str) -> Result<(String, String), CompilerError> {
    if let Some(caps) = FILENAME_RE.captures(&file) {
        match (caps.at(1), caps.at(2)) {
            (Some(src_root), Some(module)) => {
                Ok((src_root.to_string(), module.to_string()))
            },
            (None, Some(module)) => {
                Ok(("./".to_string(), module.to_string()))
            },
            (Some(_), None) | (None, None) => {
                return Err(CompilerError::IOError(format!("'{}' not found",
                    file)))
            }
        }
    } else {
        return Err(CompilerError::IOError(format!("unsupported filetype for \
            file: {}", file)))
    }
}

// Scope gathering helper functions
/// This function gathers id's that will be instantiated in the current scope
/// and orders them for the compiler to use when looking up or assigning values
pub fn gather_scope(stmts: &Vec<Statement>, start_ndx: usize)
    -> Result<HashMap<String, usize>, CompilerError> {
    let mut scope_set = HashSet::new();
    let mut scope_map = HashMap::new();

    rec_gather_scope(&mut scope_set, stmts)?;

    let end_ndx = start_ndx + scope_set.len();
    (start_ndx..end_ndx).into_iter().zip(scope_set.into_iter())
        .for_each(|(ndx, key)| {
            scope_map.insert(key, ndx);
        });

    Ok(scope_map)
}

/// Recursively identifies statements that will modify a single level of scope
fn rec_gather_scope(scope: &mut HashSet<String>, stmts: &Vec<Statement>)
    -> Result<(), CompilerError> {
    for stmt in stmts.iter() {
        match *stmt {
            Statement::FunctionDef { ref name, .. } => {
                scope.insert(name.clone());
            },
            Statement::ClassDef { ref name, .. } => {
                scope.insert(name.clone());
            },
            Statement::Assign { ref targets, .. } => {
                for target in targets.iter() {
                    unpack_assign_targets(scope, target)?;
                }
            },
            Statement::For { ref target, iter: _, ref body, ref orelse } => {
                unpack_assign_targets(scope, target)?;
                rec_gather_scope(scope, body)?;
                rec_gather_scope(scope, orelse)?;
            },
            Statement::While { test: _, ref body, ref orelse } => {
                rec_gather_scope(scope, body)?;
                rec_gather_scope(scope, orelse)?;
            },
            Statement::If { test: _, ref body, ref orelse } => {
                rec_gather_scope(scope, body)?;
                rec_gather_scope(scope, orelse)?;
            },
            Statement::Import { ref names } => {
                for name in names.iter() {
                    let (name, asname) = match *name {
                        Alias::Alias { ref name, ref asname } => (name, asname)
                    };
                    let alias = match *asname {
                        Some(ref alias) => alias,
                        None => name
                    };

                    scope.insert(alias.clone());
                }
            },
            Statement::ImportFrom { .. } => {
                // TODO, if we wanted to support import from we are going to
                // run into issues with wildcards. We would need to gather
                // the scope of the entire module.
                unimplemented!();
            },
            _ => ()
        }
    }

    Ok(())
}

pub fn gather_func_params(params: &Arguments, start_ndx: usize)
    -> Result<HashMap<String, usize>, CompilerError> {
    let mut scope_set = HashSet::new();
    let mut scope_map = HashMap::new();
    let (args, _vararg, _kwonlyargs, _kw_defaults, _kwarg, _defaults) =
    match *params {
        Arguments::Arguments { ref args, ref vararg, ref kwonlyargs,
            ref kw_defaults, ref kwarg, ref defaults } => (args, vararg,
            kwonlyargs, kw_defaults, kwarg, defaults)
    };

    for arg in args.iter() {
        let arg_name = match *arg {
            Arg::Arg { ref arg, .. } => arg
        };

        scope_set.insert(arg_name.to_string());
    }

    let end_ndx = start_ndx + scope_set.len();
    (start_ndx..end_ndx).into_iter().zip(scope_set.into_iter())
        .for_each(|(ndx, key)| {
            scope_map.insert(key, ndx);
        });

    Ok(scope_map)
}

pub fn gather_comp_targets(generators: &Vec<Comprehension>, start_ndx: usize)
    -> Result<HashMap<String, usize>, CompilerError> {
    let mut scope_set = HashSet::new();
    let mut scope_map = HashMap::new();

    let mut gen_iter = generators.iter();
    while let Some(&Comprehension::Comprehension { ref target, .. })
        = gen_iter.next() {
        unpack_assign_targets(&mut scope_set, target)?;
    }

    let end_ndx = start_ndx + scope_set.len();
    (start_ndx..end_ndx).into_iter().zip(scope_set.into_iter())
        .for_each(|(ndx, key)| {
            scope_map.insert(key, ndx);
        });

    Ok(scope_map)
}

fn unpack_assign_targets(scope: &mut HashSet<String>, target: &Expression)
    -> Result<(), CompilerError> {
    match *target {
        Expression::Name { ref id, .. } => {
            scope.insert(id.clone());
        },
        Expression::List { .. } => unimplemented!(),
        Expression::Tuple { ref elts, .. } => {
            for elt in elts.iter() {
                match *elt {
                    Expression::Name { ref id, .. } => {
                        scope.insert(id.clone());
                    },
                    Expression::Tuple { .. } => {
                        unpack_assign_targets(scope, elt)?;
                    },
                    _ => unimplemented!()
                }
            }
        },
        _ => ()
    }

    Ok(())
}

/// Traverses the compiler's scope list to find a value, if the value is found
/// a tuple (scope_position, value_offset) is returned.
pub fn lookup_value(scope: &Vec<HashMap<String, usize>>, id: &str)
    -> Result<(usize, usize), CompilerError> {
    for (ndx, tbl) in scope.iter().enumerate().rev() {
        if let Some(offset) = tbl.get(id) {
            return Ok((ndx, *offset))
        }
    }
    Err(CompilerError::NameError(id.to_string()))
}

lazy_static! {
   static ref FILENAME_RE: Regex = Regex::new(r"(.*/)?(.+)\.py$").unwrap();
}
