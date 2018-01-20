use std::fs::File;
use std::io;
use std::io::Write;

use ::compiler::cfg::operand::{Operand, Register};
use ::compiler::arch::llvm::LLVM;

#[derive(Debug)]
pub struct InvocStruct {
    pub result: Register,
    pub func_name: String,
    pub args: Vec<Operand>
}

impl LLVM for InvocStruct {
    fn output_llvm(&self, f: &mut File) -> Result<(), io::Error> {
        let func = String::from(format!("\t{} = call %struct.Type* @{}",
            self.result, self.func_name));
        let mut args = String::new();

        for arg in self.args.iter() {
            args.push_str(format!("{},", arg).as_str());
        }
        // Pop the trailing comma or get None if args is empty
        args.pop();
        write!(f, "{}({})\n", func, args)
    }
}
