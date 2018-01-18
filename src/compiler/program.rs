use super::function::Function;
use std::fs::File;
use std::io;
use std::io::Write;

pub struct Program {
    pub funcs: Vec<Function>
}

impl Program {
    pub fn output_llvm(&self, f: &mut File) -> Result<(), io::Error> {
        Program::output_forward_decls(f)?;

        for func in self.funcs.iter() {
            func.output_llvm(f)?
        }
        Ok(())
    }

    /// Forward declaration of functions from the linked library
    fn output_forward_decls(f: &mut File) -> Result<(), io::Error> {
        f.write_all("\
            declare i32 @add(i32, i32)\n\
        \n".as_bytes())
    }
}
