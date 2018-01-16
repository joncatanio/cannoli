use ::compiler::cfg::register::{Operand, Register};
use std::fs::File;;

#[derive(Debug)]
pub struct Arith {
    result: Register,
    inst: ArithOp,
    op1: Operand,
    op2: Operand
}

impl Instruction for Arith {
    fn output_llvm(&self, f: File) {
        writeln!(f, "test")
    }
}
