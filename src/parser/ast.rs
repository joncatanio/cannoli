// Abstract Syntax Tree definitions

#[derive(Debug, PartialEq)]
pub enum Ast {
    Module { body: Vec<Statement> }
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    Global { names: Vec<String> },
    Nonlocal { names: Vec<String> },
    Pass,
    Break,
    Continue,
    Return { value: Option<Expression> },
    Expr { value: Expression },
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    BoolOp { op: BoolOperator, values: Vec<Expression> },
    If { test: Box<Expression>, body: Box<Expression>,
        orelse: Box<Expression> },
    NameConstant { value: Singleton },
    Tuple { elts: Vec<Expression>, ctx: ExprContext },
}

#[derive(Debug, PartialEq)]
pub enum ExprContext {
    Load,
    Store,
    Del,
    AugLoad,
    AugStore,
    Param
}

#[derive(Debug, PartialEq)]
pub enum BoolOperator {
    And,
    Or
}

#[derive(Debug, PartialEq)]
pub enum Singleton {
    None,
    True,
    False
}
