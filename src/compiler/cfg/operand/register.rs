use std::fmt;

#[derive(Debug)]
pub struct Register {
    label: String
}

impl Register {
    pub fn new() -> Register {
        unsafe {
            static mut SUFFIX: usize = 0;
            let label = format!("r{}", SUFFIX);
            SUFFIX += 1;

            Register { label }
        }
    }

    pub fn get_label(&self) -> String {
        self.label.clone()
    }
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.label)
    }
}

impl Operand for Register {}
