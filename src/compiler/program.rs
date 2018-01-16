use super::function::Function;
use std::fs::File;
use std::fmt::Error;

pub struct Program {
    funcs: Vec<Function>
}

impl Program {
    pub fn output_llvm(&self, f: &mut File) -> Result<(), Error> {
        for func in self.funcs.iter() {
            func.output_llvm(f)?
        }
        Ok(())
    }
}
