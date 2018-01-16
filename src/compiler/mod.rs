pub mod cfg;
mod function;
mod program;

use clap::ArgMatches;
use super::parser::ast::*;
use self::function::Function;
use self::program::Program;
use self::cfg::CFG;
use self::cfg::inst;
use self::cfg::operand::{Register, Immediate};

pub fn compile(ast: Ast, _args: &ArgMatches) -> Program {
    /*
    match args.value_of("o").unwrap_or("") {
        "1" => unimplemented!(),
        "2" => unimplemented!(),
        "3" => unimplemented!(),
        _ => ()
    }
    */
    let mut funcs = gather_funcs(&ast);
    let main = gather_main(&ast);
    funcs.insert(0, main);

    Program { funcs }
}

fn gather_funcs(ast: &Ast) -> Vec<Function> {
    let body = match *ast {
        Ast::Module { ref body } => body
    };

    for stmt in body.iter() {
        match *stmt {
            Statement::FunctionDef { .. } => {
                unimplemented!()
            },
            _ => ()
        }
    }

    // TODO REMOVE
    vec![]
}

fn gather_main(ast: &Ast) -> Function {
    let body = match *ast {
        Ast::Module { ref body } => body
    };
    let mut cfg = CFG::new();
    let mut cur_block = cfg.entry_block.clone();

    for stmt in body.iter() {
        match *stmt {
            Statement::FunctionDef { .. } | Statement::ClassDef { .. } => (),
            _ => {
                cur_block = compile_stmt(&mut cfg, cur_block, stmt);
            }
        }
    }

    // Check if cur_block is equal to exit_block, if not connect them
    // TODO REMOVE BELOW
    Function { name: "main".to_string(), graph: cfg }
}

fn compile_stmts(cfg: &mut CFG, mut cur_block: String, stmts: &Vec<Statement>)
    -> String {
    for stmt in stmts.iter() {
        cur_block = compile_stmt(cfg, cur_block.clone(), stmt);
    }

    cur_block
}

fn compile_stmt(cfg: &mut CFG, cur_block: String, stmt: &Statement) -> String {
    match *stmt {
        Statement::Expr { ref value } =>  {
            compile_stmt_expr(cfg, cur_block, value)
        },
        _ => unimplemented!()
    }
}

fn compile_stmt_expr(cfg: &mut CFG, cur_block: String, expr: &Expression)
    -> String {
    let reg = match *expr {
        Expression::BinOp { .. } => compile_expr_binop(cfg, cur_block.clone(), expr),
        _ => unimplemented!()
    };

    cur_block
}

fn compile_expr_binop(cfg: &mut CFG, cur_block: String, expr: &Expression)
    -> Register {
    let (left, op, right) = match *expr {
        Expression::BinOp { ref left, ref op, ref right } => (left, op, right),
        _ => unreachable!()
    };

    // Get the resulting registers from left/right then generate the bin op
    // inst and add it to the CFG.

    // TODO REMOVE, THIS IS ONLY TEMPORARY
    let reg = Register::new();
    let inst = inst::Arith {
        result: reg.clone(), inst: "+".to_string(),
        op1: Box::new(Immediate { value: "1".to_string() }),
        op2: Box::new(Immediate { value: "2".to_string() })
    };
    cfg.add_inst(cur_block, Box::new(inst));
    reg
    // TODO REMOVE ABOVE
}
