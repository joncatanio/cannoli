use std::fs::File;
use std::io;
use std::io::Write;

use ::compiler::cfg::operand::{Operand, Register};
use super::Instruction;

pub enum ArithOp {
    Add,
    Sub,
    Mult,
    MatMult,
    Div,
    Mod,
    Pow,
    FloorDiv
}

pub struct Arith {
    pub result: Register,
    pub inst: ArithOp, // TODO change to enum or something
    pub op1: Box<Operand>,
    pub op2: Box<Operand>
}

impl Instruction for Arith {
    fn output_llvm(&self, f: &mut File) -> Result<(), io::Error> {
        match self.inst {
            ArithOp::Add => {
                write!(f, "\t%{} = call i32 @add({}, {})\n",
                    self.result, self.op1, self.op2)
            },
            _ => unimplemented!()
        }
    }

    fn debug(&self) -> String {
        format!("Arith")
    }
}
