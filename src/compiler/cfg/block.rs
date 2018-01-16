use super::inst::Instruction;

use std::fs::File;
use std::fmt::Error;

#[derive(Debug)]
pub struct Block {
    label: String,
    insts: Vec<Box<Instruction>>
}

impl Block {
    pub fn new() -> Block {
        unsafe {
            static mut SUFFIX: usize = 0;
            let label = format!("B{}", SUFFIX);
            SUFFIX += 1;

            Block { label, insts: vec![] }
        }
    }

    pub fn get_label(&self) -> String {
        self.label.clone()
    }

    pub fn output_llvm(&self, f: &mut File) -> Result<(), Error> {
        unimplemented!()
    }
}
