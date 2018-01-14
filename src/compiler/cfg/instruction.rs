use std::fmt;

pub trait Instruction {
    // TODO implement IR output functions
    fn output_llvm(&self) {}
    fn inst_info(&self) -> String;
}

impl fmt::Debug for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.inst_info())
    }
}
