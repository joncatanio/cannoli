use super::cfg::CFG;
use std::fs::File;
use std::io;

pub struct Function {
    // TODO add `params` and other meta information
    pub name: String,
    pub graph: CFG
}

impl Function {
    pub fn output_llvm(&self, f: &mut File) -> Result<(), io::Error> {
        self.graph.output_llvm(f)
    }
}
