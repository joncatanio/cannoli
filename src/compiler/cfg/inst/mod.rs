pub mod arith;
pub use self::arith::{ArithStruct, ArithOp};
pub mod invoc;
pub use self::invoc::InvocStruct;
pub mod alloc;
pub use self::alloc::AllocStruct;
pub mod rtn;
pub use self::rtn::ReturnStruct;
pub mod branch;
pub use self::branch::BranchStruct;

use ::compiler::arch::llvm::LLVM;
use std::fs::File;
use std::io;

#[derive(Debug)]
pub enum Instruction {
    Arith(ArithStruct),
    Invoc(InvocStruct),
    Alloc(AllocStruct),
    Return(ReturnStruct),
    Branch(BranchStruct)
}

impl LLVM for Instruction {
    fn output_llvm(&self, f: &mut File) -> Result<(), io::Error> {
        match *self {
            Instruction::Arith(ref inst)  => inst.output_llvm(f),
            Instruction::Invoc(ref inst)  => inst.output_llvm(f),
            Instruction::Alloc(ref inst)  => inst.output_llvm(f),
            Instruction::Return(ref inst) => inst.output_llvm(f),
            Instruction::Branch(ref inst) => inst.output_llvm(f)
        }
    }
}
