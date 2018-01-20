/// Utility functions to aid in generating CFG block instructions
use ::parser::ast::*;
use super::types;
use super::cfg::CFG;
use super::cfg::inst::*;
use super::cfg::operand::{Operand, Register, Immediate};

/// Inserts the proper invocation instruction in the appropriate CFG block
/// that properly constructs a library type
pub fn construct_type(cfg: &mut CFG, cur_block: String, op: &Operand)
    -> Operand {
    // TODO figure out how I want to insert the construct instructions
    match *op {
        Operand::Reg(ref reg) => {
            unimplemented!()
        },
        Operand::Imm(ref imm) => {
            match imm.value {
                types::Type::Str(ref s) => {
                    unimplemented!()
                },
                types::Type::Num(ref n) => {
                    let reg = Register::new();
                    unimplemented!()
                },
                types::Type::Object => unimplemented!()
            }
        }
    }
}

pub fn gen_imm_num(num: &Number) -> Operand {
    // TODO change this to reflect the original type in the Immediate struct
    let value = match *num {
        Number::DecInteger(ref s) => {
            types::Type::Num(types::Number::DecInteger(s.clone()))
        },
        Number::BinInteger(ref s) => {
            types::Type::Num(types::Number::BinInteger(s.clone()))
        },
        Number::OctInteger(ref s) => {
            types::Type::Num(types::Number::OctInteger(s.clone()))
        },
        Number::HexInteger(ref s) => {
            types::Type::Num(types::Number::HexInteger(s.clone()))
        },
        Number::Float(ref s)      => {
            types::Type::Num(types::Number::Float(s.clone()))
        },
        Number::Imaginary(ref s)  => {
            types::Type::Num(types::Number::Imaginary(s.clone()))
        }
    };

    Operand::Imm(Immediate::new(value))
}

pub fn gen_bin_inst(cfg: &mut CFG, cur_block: String, op: &Operator,
    lft_oper: Operand, rht_oper: Operand) -> Operand {
    match *op {
        Operator::Add =>
            gen_arith_inst(cfg, cur_block, ArithOp::Add, lft_oper, rht_oper),
        _ => unimplemented!()
    }
}

pub fn gen_arith_inst(cfg: &mut CFG, cur_block: String, op: ArithOp,
    lft_oper: Operand, rht_oper: Operand) -> Operand {
    let reg = Register::new();
    let inst = Instruction::Arith(ArithStruct { result: reg.clone(), inst: op,
        op1: lft_oper, op2: rht_oper });

    cfg.add_inst(cur_block, inst);
    Operand::Reg(reg)
}
