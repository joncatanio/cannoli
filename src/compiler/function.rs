use super::cfg::CFG;
use std::fs::File;
use std::fmt::Error;

pub struct Function {
    // TODO add `params` and other meta information
    name: String,
    graph: CFG
}

impl Function {
    pub fn output_llvm(&self, f: &mut File) -> Result<(), Error> {
        self.graph.output_llvm(f)
    }
}
