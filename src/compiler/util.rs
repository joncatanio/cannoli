/// Utility functions to aid in generating CFG block instructions
use ::parser::ast::*;
use super::types;
use super::cfg::CFG;
use super::cfg::inst::*;
use super::cfg::operand::{Operand, Register, Immediate};

pub fn gen_bin_inst(cfg: &mut CFG, cur_block: String, op: &Operator,
    lft_oper: Box<Operand>, rht_oper: Box<Operand>) -> Box<Operand> {
    match *op {
        Operator::Add =>
            gen_arith_inst(cfg, cur_block, ArithOp::Add, lft_oper, rht_oper),
        _ => unimplemented!()
    }
}

pub fn gen_arith_inst(cfg: &mut CFG, cur_block: String, op: ArithOp,
    lft_oper: Box<Operand>, rht_oper: Box<Operand>) -> Box<Operand> {
    let reg = Register::new();
    let inst = Arith { result: reg.clone(), inst: op,
        op1: lft_oper, op2: rht_oper };

    cfg.add_inst(cur_block, Box::new(inst));
    Box::new(reg)
}

pub fn gen_imm_num(num: &Number) -> Box<Operand> {
    // TODO change this to reflect the original type in the Immediate struct
    let value = match *num {
        Number::DecInteger(ref s) => {
            types::Type::Num { n: types::Number::DecInteger(s.clone()) }
        },
        Number::BinInteger(ref s) => {
            types::Type::Num { n: types::Number::BinInteger(s.clone()) }
        },
        Number::OctInteger(ref s) => {
            types::Type::Num { n: types::Number::OctInteger(s.clone()) }
        },
        Number::HexInteger(ref s) => {
            types::Type::Num { n: types::Number::HexInteger(s.clone()) }
        },
        Number::Float(ref s)      => {
            types::Type::Num { n: types::Number::Float(s.clone()) }
        },
        Number::Imaginary(ref s)  => {
            types::Type::Num { n: types::Number::Imaginary(s.clone()) }
        }
    };

    Box::new(Immediate::new(value))
}
