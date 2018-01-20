use std::fmt;
use ::compiler::types::*;

#[derive(Debug)]
pub struct Immediate {
    value: Type,
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
