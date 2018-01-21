/// LLVM local variable representation
use std::fmt;

// For now locals will not have a type field because every local will
// be of type "Type*" which will be handled by the C lib.
#[derive(Debug, Clone)]
pub struct Local {
    pub label: String
}

impl Local {
    pub fn new() -> Local {
        unsafe {
            static mut SUFFIX: usize = 0;
            let label = format!("r{}", SUFFIX);
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
        write!(f, "%{}", self.label)
    }
}
