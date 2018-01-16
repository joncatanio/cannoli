use std::fs::File;
use std::io;
use std::io::Write;

use ::compiler::cfg::operand::{Operand, Register};
use super::Instruction;

pub struct Arith {
    pub result: Register,
    pub inst: String, // TODO change to enum or something
    pub op1: Box<Operand>,
    pub op2: Box<Operand>
}

impl Instruction for Arith {
    fn output_llvm(&self, f: &mut File) -> Result<(), io::Error> {
        f.write_all(b"test\n")
    }

    fn debug(&self) -> String {
        format!("Arith")
    }
}
