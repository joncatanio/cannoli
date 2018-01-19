use std::fs::File;
use std::io;

pub trait LLVM {
    fn output_llvm(&self, f: &mut File) -> Result<(), io::Error>;
}
