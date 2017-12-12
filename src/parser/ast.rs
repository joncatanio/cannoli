// Abstract Syntax Tree definitions

#[derive(Debug)]
pub enum Statement {
    SimpleStatement,
    CompoundStatement
}

#[derive(Debug)]
pub enum FileInput {
    Statements(Vec<Statement>)
}

#[derive(Debug)]
pub enum Ast {
    // single_input and eval_input are left out
    File(Vec<FileInput>)
}
