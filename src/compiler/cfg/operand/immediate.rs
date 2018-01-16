use std::fmt;
use super::Operand;

#[derive(Debug)]
pub struct Immediate {
    pub value: String
}

impl Immediate {
    pub fn new(value: String) -> Immediate {
        Immediate { value }
    }
}

impl fmt::Display for Immediate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Operand for Immediate {
    fn display(&self) -> String {
        format!("{}", self.value)
    }
}
