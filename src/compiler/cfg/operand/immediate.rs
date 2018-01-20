use std::fmt;
use ::compiler::types::*;

#[derive(Debug, Clone)]
pub struct Immediate {
    pub value: Type
}

impl Immediate {
    pub fn new(value: Type) -> Immediate {
        Immediate { value }
    }
}

impl fmt::Display for Immediate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}
