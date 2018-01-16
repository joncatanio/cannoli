pub mod register;
pub mod immediate;

pub use self::register::Register;
pub use self::immediate::Immediate;

use std::fmt;

pub trait Operand {
    fn display(&self) -> String;
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.display())
    }
}
