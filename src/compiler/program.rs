use super::function::Function;
use std::fs::File;
use std::io;

pub struct Program {
    pub funcs: Vec<Function>
}

impl Program {
    pub fn output_llvm(&self, f: &mut File) -> Result<(), io::Error> {
        for func in self.funcs.iter() {
            func.output_llvm(f)?
        }
        Ok(())
    }
}
