use super::inst::Instruction;

use std::fs::File;
use std::io;
use std::io::Write;

#[derive(Debug)]
pub struct Block {
    label: String,
    insts: Vec<Instruction>
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

    pub fn add_inst(&mut self, inst: Instruction) {
        self.insts.push(inst)
    }

    pub fn output_llvm(&self, f: &mut File) -> Result<(), io::Error> {
        f.write_all(format!("{}:\n", &self.label).as_bytes())?;
        for inst in self.insts.iter() {
            inst.output_llvm(f)?;
        }
        Ok(())
    }
}
