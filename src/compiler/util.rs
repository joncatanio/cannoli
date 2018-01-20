/// Utility functions to aid in generating CFG block instructions
use ::parser::ast::*;
use super::types;
use super::cfg::CFG;
use super::cfg::inst::*;
use super::cfg::operand::{Operand, Register, Immediate};

/// Consumes the given operand returning the Operand::Reg that represents the
/// return value of the library call to construct a Type* value.
pub fn construct_type(cfg: &mut CFG, cur_block: String, op: Operand)
    -> Operand {
    // Move from borrowed context in match is not playing nice.
    let cloned_op = op.clone();

    match op {
        Operand::Reg(_) => op,
        Operand::Imm(ref imm) => {
            match imm.value {
                types::Type::Str(_) => {
                    unimplemented!()
                },
                types::Type::Num(ref n) => {
                    let func_name = match *n {
                        types::Number::DecInteger(_) => "cons_int".to_string(),
                        types::Number::BinInteger(_) => "cons_int".to_string(),
                        types::Number::OctInteger(_) => "cons_int".to_string(),
                        types::Number::HexInteger(_) => "cons_int".to_string(),
                        types::Number::Float(_) => unimplemented!(),
                        types::Number::Imaginary(_) => unimplemented!()
                    };
                    // Construct the type system Type* value that the library
                    // can internally represent and return the associated reg.
                    gen_invoc_inst(cfg, cur_block, func_name, vec![cloned_op])
                },
                types::Type::Object => unimplemented!()
            }
        }
    }
}

pub fn gen_imm_num(cfg: &mut CFG, cur_block: String, num: &Number)
    -> Operand {
    // TODO convert the bin/oct/hex ints to integers here
    let val = match *num {
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
    let op = Operand::Imm(Immediate::new(val));

    construct_type(cfg, cur_block, op)
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

// TODO add in a rtn_type parameter that will be populated if a user gives
// a type hint in the function declaration signature. For now, we assume that
// the return type of every function will be the Type* value in the library
pub fn gen_invoc_inst(cfg: &mut CFG, cur_block: String, func_name: String,
    args: Vec<Operand>) -> Operand {
    let reg = Register::new();
    let inst = Instruction::Invoc(InvocStruct { result: reg.clone(),
        func_name, args });

    cfg.add_inst(cur_block, inst);
    Operand::Reg(reg)
}
