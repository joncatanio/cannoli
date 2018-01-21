use std::fs::File;
use std::io;
use std::io::Write;

use ::compiler::cfg::operand::Operand;
use ::compiler::arch::llvm::LLVM;

#[derive(Debug)]
pub struct BranchStruct {
    pub cond: Option<Operand>,
    pub dest1: String,
    pub dest2: Option<String>
}

impl BranchStruct {
    pub fn new(cond: Option<Operand>, dest1: String, dest2: Option<String>)
        -> BranchStruct {
        BranchStruct { cond, dest1, dest2 }
    }
}

impl LLVM for BranchStruct {
    fn output_llvm(&self, f: &mut File) -> Result<(), io::Error> {
        if self.cond.is_none() {
            write!(f, "\tbr label %{}\n", self.dest1)
        } else {
            unimplemented!()
        }
    }
}
