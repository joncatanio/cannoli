// Abstract Syntax Tree definitions

#[derive(Debug)]
pub enum Declaration {}

#[derive(Debug)]
pub enum SmallStatement {
    ExprStatement,
    DelStatement,
    PassStatement,
    FlowStatement,
    ImportStatement,
    GlobalStatement(Vec<String>),
    NonlocalStatement(Vec<String>),
    AssertStatement
}

#[derive(Debug)]
pub enum Statement {
    SimpleStatement(Vec<SmallStatement>),
    CompoundStatement
}

#[derive(Debug)]
pub enum Ast {
    // single_input and eval_input are left out
    FileInput(Vec<Statement>)
}
