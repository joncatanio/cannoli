use std::collections::HashMap;

pub type SymTbl = HashMap<String, Symbol>;

pub enum Symbol {
    Function,
    Variable
}
