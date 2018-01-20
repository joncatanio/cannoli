use std::fs::File;
use std::io;
use std::io::Write;

use ::compiler::cfg::operand::{Operand, Register};
use ::compiler::arch::llvm::LLVM;

#[derive(Debug)]
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

#[derive(Debug)]
pub struct ArithStruct {
    pub result: Register,
    pub inst: ArithOp, // TODO change to enum or something
    pub op1: Operand,
    pub op2: Operand
}

impl LLVM for ArithStruct {
    fn output_llvm(&self, f: &mut File) -> Result<(), io::Error> {
        match self.inst {
            ArithOp::Add => {
                write!(f, "\t{} = call %struct.Type* @add(%struct.Type* {}, \
                    %struct.Type* {})\n", self.result, self.op1, self.op2)
            },
            _ => unimplemented!()
        }
    }
}
