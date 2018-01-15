use std::fmt;

/// Each instruction should implement the Instruction trait in order to be
/// included in a CFG block
pub trait Instruction {
    /// Outputs the LLVM instruction represented by the inst
    fn output_llvm(&self) {}
    /// In order to implement Debug for this trait, each instruction must
    /// provide information about it's implementation
    fn inst_info(&self) -> String;
}

impl fmt::Debug for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.inst_info())
    }
}
