use super::cfg::CFG;
use std::fs::File;
use std::io;
use std::io::Write;

pub struct Function {
    // TODO add `params` and other meta information
    pub name: String,
    pub return_type: String,
    pub graph: CFG
}

impl Function {
    pub fn output_llvm(&self, f: &mut File) -> Result<(), io::Error> {
        f.write_all(format!("define {} @{} {{\n", &self.return_type,
            &self.name).as_bytes())?;
        self.graph.output_llvm(f);
        f.write_all(format!("}}\n").as_bytes())
    }
}
