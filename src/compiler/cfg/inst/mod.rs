use std::fmt;

/// Each instruction should implement the Instruction trait in order to be
/// included in a CFG block
pub trait Instruction {
    /// Outputs the LLVM instruction represented by the inst
    fn output_llvm(&self) {}
}

impl fmt::Debug for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt(f)
    }
}
