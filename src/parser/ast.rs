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
    Call { func: Box<Expression>, args: Vec<Expression>,
        keywords: Vec<Keyword> },
    Num { n: Number },
    Str { s: String },
    NameConstant { value: Singleton },
    Ellipsis,
    Attribute { value: Box<Expression>, attr: String, ctx: ExprContext },
    Subscript { value: Box<Expression>, slice: Slice, ctx: ExprContext },
    Name { id: String, ctx: ExprContext },
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
pub enum Slice {
    Slice { lower: Option<Box<Expression>>, upper: Option<Box<Expression>>,
        step: Option<Box<Expression>> },
    ExtSlice { dims: Vec<Slice> },
    Index { value: Box<Expression> }
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
pub enum Keyword {
    Keyword { arg: Option<String>, value: Expression }
}

#[derive(Debug, PartialEq)]
pub enum Singleton {
    None,
    True,
    False
}

#[derive(Debug, PartialEq)]
pub enum Number {
    DecInteger(String),
    BinInteger(String),
    OctInteger(String),
    HexInteger(String),
    Float(String),
    Imaginary(String)
}
