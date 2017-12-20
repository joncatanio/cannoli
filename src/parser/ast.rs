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
    BinOp { left: Box<Expression>, op: Operator, right: Box<Expression> },
    UnaryOp { op: UnaryOperator, operand: Box<Expression> },
    If { test: Box<Expression>, body: Box<Expression>,
        orelse: Box<Expression> },
    Compare { left: Box<Expression>, ops: Vec<CmpOperator>,
        comparators: Vec<Expression> },
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
pub enum Operator {
    Add,
    Sub,
    Mult,
    MatMult,
    Div,
    Mod,
    Pow,
    LShift,
    RShift,
    BitOr,
    BitXor,
    BitAnd,
    FloorDiv
}

#[derive(Debug, PartialEq)]
pub enum UnaryOperator {
    Invert,
    Not,
    UAdd,
    USub
}

#[derive(Debug, PartialEq)]
pub enum CmpOperator {
    EQ,
    NE,
    LT,
    LE,
    GT,
    GE,
    Is,
    IsNot,
    In,
    NotIn
}

#[derive(Debug, PartialEq)]
pub enum Singleton {
    None,
    True,
    False
}
