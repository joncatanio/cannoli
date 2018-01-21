pub mod local;
pub mod immediate;

pub use self::local::Local;
pub use self::immediate::Immediate;

use std::fmt;

#[derive(Debug, Clone)]
pub enum Operand {
    Reg(Local),
    Imm(Immediate)
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Operand::Reg(ref r) => write!(f, "{}", r),
            Operand::Imm(ref i) => write!(f, "{}", i)
        }
    }
}
