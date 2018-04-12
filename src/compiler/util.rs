use regex::Regex;
use std::collections::HashMap;

use ::parser::ast::*;
use super::errors::CompilerError;

#[derive(Debug, Clone)]
pub struct TrackedScope {
    // Vec<HashMap<identifier, (ndx, Option<type>)>>, might consider
    // changing Option<String> to Option<Value>.
    program_scope: Vec<HashMap<String, (usize, Option<String>)>>,
    // HashMap<class_name, HashMap<member_name, (ndx, Option<type>)>>
    class_map: HashMap<String, HashMap<String, (usize, Option<String>)>>
}

impl TrackedScope {
    pub fn new() -> TrackedScope {
        TrackedScope { program_scope: Vec::new(), class_map: HashMap::new() }
    }

    pub fn push_scope(&mut self, scope: HashMap<String,
        (usize, Option<String>)>) {
        self.program_scope.push(scope)
    }

    pub fn pop_scope(&mut self) -> Option<HashMap<String,
        (usize, Option<String>)>> {
        self.program_scope.pop()
    }

    pub fn insert_class(&mut self, class_name: &str, class_map:
        HashMap<String, (usize, Option<String>)>) {
        self.class_map.insert(class_name.to_string(), class_map);
    }

    /// Traverses the compiler's scope list to find a value. If the value is
    /// found a tuple (scope_position, value_offset, type) is returned.
    pub fn lookup_value(&self, id: &str)
        -> Result<(usize, usize, Option<String>), CompilerError> {
        for (ndx, tbl) in self.program_scope.iter().enumerate().rev() {
            if let Some(&(ref offset, ref vtype)) = tbl.get(id) {
                return Ok((ndx, *offset, vtype.clone()))
            }
        }
        Err(CompilerError::NameError(id.to_string()))
    }

    /// Same as 'lookup_value' but takes an Expression and returns an option
    /// in case it can't figure out the expression mapping
    pub fn lookup_expr(&self, expr: &Expression)
        -> Option<(usize, usize, Option<String>)> {
        let id = match *expr {
            Expression::Name { ref id, .. } => id,
            _ => return None
        };

        for (ndx, tbl) in self.program_scope.iter().enumerate().rev() {
            if let Some(&(ref offset, ref vtype)) = tbl.get(id) {
                return Some((ndx, *offset, vtype.clone()))
            }
        }
        None
    }

    /// Lookups up an attribute index for a given class
    pub fn lookup_attr(&self, class_name: &str, attr: &str)
        -> Result<usize, CompilerError> {
        let class_tbl = match self.class_map.get(class_name) {
            Some(tbl) => tbl,
            None => return Err(CompilerError::NameError(class_name.to_string()))
        };

        match class_tbl.get(attr) {
            Some(&(ref offset, _)) => Ok(*offset),
            None => return Err(CompilerError::AttributeError(
                class_name.to_string(), attr.to_string()))
        }
    }

    /// Similar to 'lookup_value' but annotates the value before returning it
    pub fn annotate(&mut self, id: &str, annotation: &Expression)
        -> Result<(usize, usize, Option<String>), CompilerError> {
        // TODO support more complex annotations
        let annotated_type = match *annotation {
            Expression::Name { ref id, .. } => Some(id.to_string()),
            _ => unimplemented!()
        };

        for (ndx, tbl) in self.program_scope.iter_mut().enumerate().rev() {
            let offset = match tbl.get(id) {
                Some(&(ref offset, _)) => Some(*offset),
                None => None
            };

            if let Some(offset) = offset {
                *tbl.get_mut(id).unwrap() = (offset, annotated_type.clone());
                return Ok((ndx, offset, annotated_type))
            }
        }
        Err(CompilerError::NameError(id.to_string()))
    }
}

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
pub fn gather_scope(stmts: &Vec<Statement>, start_ndx: usize, is_class: bool)
    -> Result<HashMap<String, (usize, Option<String>)>, CompilerError> {
    let mut scope_map = HashMap::new();
    let mut map = HashMap::new();

    rec_gather_scope(&mut scope_map, stmts, is_class)?;

    let end_ndx = start_ndx + scope_map.len();
    (start_ndx..end_ndx).into_iter().zip(scope_map.into_iter())
        .for_each(|(ndx, key)| {
            map.insert(key.0, (ndx, key.1));
        });

    Ok(map)
}

/// Recursively identifies statements that will modify a single level of scope
fn rec_gather_scope(scope: &mut HashMap<String, Option<String>>,
    stmts: &Vec<Statement>, is_class: bool) -> Result<(), CompilerError> {
    for stmt in stmts.iter() {
        match *stmt {
            Statement::FunctionDef { ref name, .. } => {
                scope.insert(name.clone(), None);

                if is_class && name == "__init__" {
                    gather_class_init(scope, stmt)?;
                }
            },
            Statement::ClassDef { ref name, .. } => {
                scope.insert(name.clone(), Some(name.to_string()));
            },
            Statement::Assign { ref targets, .. } => {
                for target in targets.iter() {
                    unpack_assign_targets(scope, target)?;
                }
            },
            Statement::AnnAssign { ref target, .. } => {
                unpack_assign_targets(scope, target)?;
            },
            Statement::For { ref target, iter: _, ref body, ref orelse } => {
                unpack_assign_targets(scope, target)?;
                rec_gather_scope(scope, body, is_class)?;
                rec_gather_scope(scope, orelse, is_class)?;
            },
            Statement::While { test: _, ref body, ref orelse } => {
                rec_gather_scope(scope, body, is_class)?;
                rec_gather_scope(scope, orelse, is_class)?;
            },
            Statement::If { test: _, ref body, ref orelse } => {
                rec_gather_scope(scope, body, is_class)?;
                rec_gather_scope(scope, orelse, is_class)?;
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

                    scope.insert(alias.clone(), None);
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
    -> Result<HashMap<String, (usize, Option<String>)>, CompilerError> {
    let mut scope_map = HashMap::new();
    let mut map = HashMap::new();
    let (args, _vararg, _kwonlyargs, _kw_defaults, _kwarg, _defaults) =
    match *params {
        Arguments::Arguments { ref args, ref vararg, ref kwonlyargs,
            ref kw_defaults, ref kwarg, ref defaults } => (args, vararg,
            kwonlyargs, kw_defaults, kwarg, defaults)
    };

    for arg in args.iter() {
        let (arg_name, annotation) = match *arg {
            Arg::Arg { ref arg, ref annotation } => (arg, annotation)
        };

        if let &Some(Expression::Name { ref id, .. }) = annotation {
            scope_map.insert(arg_name.to_string(), Some(id.to_string()));
        } else {
            scope_map.insert(arg_name.to_string(), None);
        }
    }

    let end_ndx = start_ndx + scope_map.len();
    (start_ndx..end_ndx).into_iter().zip(scope_map.into_iter())
        .for_each(|(ndx, key)| {
            map.insert(key.0, (ndx, key.1));
        });

    Ok(map)
}

pub fn gather_comp_targets(generators: &Vec<Comprehension>, start_ndx: usize)
    -> Result<HashMap<String, (usize, Option<String>)>, CompilerError> {
    let mut scope_map = HashMap::new();
    let mut map = HashMap::new();

    let mut gen_iter = generators.iter();
    while let Some(&Comprehension::Comprehension { ref target, .. })
        = gen_iter.next() {
        unpack_assign_targets(&mut scope_map, target)?;
    }

    let end_ndx = start_ndx + scope_map.len();
    (start_ndx..end_ndx).into_iter().zip(scope_map.into_iter())
        .for_each(|(ndx, key)| {
            map.insert(key.0, (ndx, key.1));
        });

    Ok(map)
}

/// Should only be called on __init__ functions to gather the proper class
/// initialization identifiers.
fn gather_class_init(scope: &mut HashMap<String, Option<String>>,
    func: &Statement) -> Result<(), CompilerError> {
    let (args, body) = match *func {
        Statement::FunctionDef { ref name, ref args, ref body, .. } => {
            if name != "__init__" {
                panic!("'gather_class_init' may only be called on '__init__'")
            }

            match *args {
                Arguments::Arguments { ref args, .. } => (args, body)
            }
        },
        _ => unreachable!()
    };
    let self_alias = if args.len() > 0 {
        match args[0] {
            Arg::Arg { ref arg, .. } => arg
        }
    } else {
        // return since they might be using __init__ in an irregular way
        return Ok(())
    };

    rec_gather_class_init(scope, body, self_alias)?;

    Ok(())
}

fn rec_gather_class_init(scope: &mut HashMap<String, Option<String>>,
    stmts: &Vec<Statement>, self_alias: &str) -> Result<(), CompilerError> {
    for stmt in stmts.iter() {
        match *stmt {
            Statement::Assign { ref targets, .. } => {
                for target in targets.iter() {
                    unpack_assign_alias(scope, target, self_alias)?;
                }
            },
            Statement::AnnAssign { ref target, .. } => {
                unpack_assign_alias(scope, target, self_alias)?;
            },
            Statement::For { ref body, ref orelse, .. } => {
                rec_gather_class_init(scope, body, self_alias)?;
                rec_gather_class_init(scope, orelse, self_alias)?;
            },
            Statement::While { test: _, ref body, ref orelse } => {
                rec_gather_class_init(scope, body, self_alias)?;
                rec_gather_class_init(scope, orelse, self_alias)?;
            },
            Statement::If { test: _, ref body, ref orelse } => {
                rec_gather_class_init(scope, body, self_alias)?;
                rec_gather_class_init(scope, orelse, self_alias)?;
            },
            _ => ()
        }
    }

    Ok(())
}

fn unpack_assign_targets(scope: &mut HashMap<String, Option<String>>,
    target: &Expression) -> Result<(), CompilerError> {
    match *target {
        Expression::Name { ref id, .. } => {
            scope.insert(id.clone(), None);
        },
        Expression::List { .. } => unimplemented!(),
        Expression::Tuple { ref elts, .. } => {
            for elt in elts.iter() {
                unpack_assign_targets(scope, elt)?;
            }
        },
        _ => ()
    }

    Ok(())
}

/// This function adds all assignment attributes for a given alias. The
/// main example of this would be collecing 'self.*' assignments in the
/// __init__() method for a class definition
fn unpack_assign_alias(scope: &mut HashMap<String, Option<String>>,
    target: &Expression, alias: &str) -> Result<(), CompilerError> {
    match *target {
        Expression::Attribute { ref value, ref attr, .. } => {
            let name = match **value {
                Expression::Name { ref id, .. } => id,
                _ => return Ok(())
            };

            if name == alias {
                scope.insert(attr.clone(), None);
            }
        },
        Expression::List { .. } => unimplemented!(),
        Expression::Tuple { ref elts, .. } => {
            for elt in elts.iter() {
                unpack_assign_alias(scope, elt, alias)?;
            }
        },
        _ => ()
    }

    Ok(())
}

lazy_static! {
   static ref FILENAME_RE: Regex = Regex::new(r"(.*/)?(.+)\.py$").unwrap();
}
