use std::fmt;

#[derive(Debug, Clone)]
pub struct Local {
    label: String
}

impl Local {
    pub fn new() -> Local {
        unsafe {
            static mut SUFFIX: usize = 0;
            let label = format!("v{}", SUFFIX);
            SUFFIX += 1;

            Local { label }
        }
    }

    pub fn get_label(&self) -> String {
        self.label.clone()
    }
}

impl fmt::Display for Local {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.get_label())
    }
}
