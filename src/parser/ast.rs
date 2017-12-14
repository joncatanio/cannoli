// Abstract Syntax Tree definitions

#[derive(Debug)]
pub enum Ast {
    Module { body: Vec<Statement> }
}

#[derive(Debug)]
pub enum Statement {
    GlobalStatement { names: Vec<String> },
    NonlocalStatement { names: Vec<String> },
    PassStatement,
    BreakStatement,
    ContinueStatement,
    ReturnStatement { value: Option<Expression> }
}

#[derive(Debug)]
pub enum Expression {
    NumExpression(usize)
}
