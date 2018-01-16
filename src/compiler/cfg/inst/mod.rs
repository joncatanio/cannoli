pub mod arith;
pub use self::arith::Arith;

use std::fs::File;
use std::fmt;
use std::io;

/// Each instruction should implement the Instruction trait in order to be
/// included in a CFG block
pub trait Instruction {
    /// Outputs the LLVM instruction represented by the inst
    fn output_llvm(&self, f: &mut File) -> Result<(), io::Error>;
    fn debug(&self) -> String;
}

impl fmt::Debug for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.debug())
    }
}
