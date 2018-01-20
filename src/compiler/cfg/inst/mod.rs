pub mod arith;
pub use self::arith::{ArithStruct, ArithOp};

use ::compiler::arch::llvm::LLVM;
use std::fs::File;
use std::io;

#[derive(Debug)]
pub enum Instruction {
    Arith(ArithStruct)
}

impl LLVM for Instruction {
    fn output_llvm(&self, f: &mut File) -> Result<(), io::Error> {
        match *self {
            Instruction::Arith(ref inst) => inst.output_llvm(f)
        }
    }
}
