// Abstract Syntax Tree definitions

#[derive(Debug)]
pub enum Expression {
    NumExpression(usize)
}

#[derive(Debug)]
pub enum Statement {
    // Main Statement
    SimpleStatement(Vec<Statement>),
    CompoundStatement,
    // Small Statements
    ExprStatement,
    DelStatement,
    PassStatement,
    ImportStatement,
    GlobalStatement(Vec<String>),
    NonlocalStatement(Vec<String>),
    AssertStatement,
    // Flow Statements
    BreakStatement,
    ContinueStatement,
    ReturnStatement(Option<Expression>)
}

#[derive(Debug)]
pub enum Ast {
    // single_input and eval_input are left out
    FileInput(Vec<Statement>)
}
