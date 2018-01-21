use std::fs::File;
use std::io;
use std::io::Write;

use ::compiler::cfg::operand::Local;
use ::compiler::arch::llvm::LLVM;

#[derive(Debug)]
pub struct AllocStruct {
    pub result: Local
}

impl LLVM for AllocStruct {
    fn output_llvm(&self, f: &mut File) -> Result<(), io::Error> {
        write!(f, "{} = alloca %struct.Type*", self.result)
    }
}
