use std::fs::File;
use std::io;
use std::io::Write;

use ::compiler::cfg::operand::Operand;
use ::compiler::arch::llvm::LLVM;

// TODO need to figure out something clever to do with the return instruction,
// immediates will print out their type but registers don't, the return_type
// for most functions (except main) should be Type*
#[derive(Debug)]
pub struct ReturnStruct {
    pub return_type: String,
    pub value: Option<Operand>
}

impl LLVM for ReturnStruct {
    fn output_llvm(&self, f: &mut File) -> Result<(), io::Error> {
        if self.value.is_none() {
            write!(f, "\tret void\n")
        } else {
            unimplemented!()
        }
    }
}
