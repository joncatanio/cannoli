use std::fmt;

#[derive(Debug)]
pub struct Immediate {
    value: String
}

impl Immediate {
    pub fn new(value: String) -> Immediate {
        Immediate { value }
    }
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Operand for Immediate {}
