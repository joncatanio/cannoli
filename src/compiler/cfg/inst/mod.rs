pub mod arith;
pub use self::arith::{ArithStruct, ArithOp};
pub mod invoc;
pub use self::invoc::InvocStruct;

use ::compiler::arch::llvm::LLVM;
use std::fs::File;
use std::io;

#[derive(Debug)]
pub enum Instruction {
    Arith(ArithStruct),
    Invoc(InvocStruct)
}

impl LLVM for Instruction {
    fn output_llvm(&self, f: &mut File) -> Result<(), io::Error> {
        match *self {
            Instruction::Arith(ref inst) => inst.output_llvm(f),
            Instruction::Invoc(ref inst) => inst.output_llvm(f)
        }
    }
}
