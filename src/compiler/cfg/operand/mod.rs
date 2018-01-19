pub mod register;
pub mod immediate;

pub use self::register::Register;
pub use self::immediate::Immediate;

use std::fmt;

#[derive(Debug)]
pub enum Operand {
    Reg(Register),
    Imm(Immediate)
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Reg(r) => write!(f, "{}", r),
            Imm(i) => write!(f, "{}", i)
        }
    }
}
