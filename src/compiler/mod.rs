pub mod cfg;
mod function;
mod program;

use clap::ArgMatches;
use super::parser::ast::*;
use self::function::Function;
use self::cfg::CFG;
use self::cfg::block::Block;

pub fn compile(ast: Ast, _args: &ArgMatches) {
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

    unimplemented!()
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

    unimplemented!()
}

fn gather_main(ast: &Ast) -> Function {
    let body = match *ast {
        Ast::Module { ref body } => body
    };
    let cfg = CFG::new();
    let cur_block = compile_stmts(&cfg, cfg.entry_block.clone(), body);

    // Check if cur_block is equal to exit_block, if not connect them
    unimplemented!()
}

fn compile_stmts(cfg: &CFG, mut cur_block: String, stmts: &Vec<Statement>)
    -> String {
    for stmt in stmts.iter() {
        match *stmt {
            Statement::FunctionDef { .. } | Statement::ClassDef { .. } => (),
            _ => {
                cur_block = compile_stmt(&cfg, cur_block, stmt);
            }
        }
    }

    cur_block
}

fn compile_stmt(cfg: &CFG, cur_block: String, stmt: &Statement) -> String {
    match *stmt {
        Statement::Expr { ref value } =>  {
            compile_stmt_expr(cfg, cur_block, value)
        },
        _ => unimplemented!()
    }
}

fn compile_stmt_expr(cfg: &CFG, cur_block: String, expr: &Expression)
    -> String {
    match *expr {
        Expression::BinOp { .. } => compile_expr_binop(cfg, cur_block, expr),
        _ => unimplemented!()
    }
}

fn compile_expr_binop(cfg: &CFG, cur_block: String, expr: &Expression)
    -> String {
    let (left, op, right) = match *expr {
        Expression::BinOp { ref left, ref op, ref right } => (left, op, right),
        _ => unreachable!()
    };

    // Get the resulting registers from left/right then generate the bin op
    // inst and add it to the CFG.
    unimplemented!()
}
