// Abstract Syntax Tree definitions
// TODO implement trait or something to add line/col num & other meta info

#[derive(Debug, PartialEq, Clone)]
pub enum Ast {
    Module { body: Vec<Statement> }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    FunctionDef { name: String, args: Arguments, body: Vec<Statement>,
        decorator_list: Vec<Expression>, returns: Option<Expression> },
    ClassDef { name: String, bases: Vec<Expression>, keywords: Vec<Keyword>,
        body: Vec<Statement>, decorator_list: Vec<Expression> },
    Return { value: Option<Expression> },
    Delete { targets: Vec<Expression> },
    Assign { targets: Vec<Expression>, value: Expression },
    AugAssign { target: Expression, op: Operator, value: Expression },
    AnnAssign { target: Expression, annotation: Expression,
        value: Option<Expression> },
    For { target: Expression, iter: Expression, body: Vec<Statement>,
        orelse: Vec<Statement> },
    While { test: Expression, body: Vec<Statement>, orelse: Vec<Statement> },
    If { test: Expression, body: Vec<Statement>, orelse: Vec<Statement> },
    With { items: Vec<WithItem>, body: Vec<Statement> },
    Raise { exc: Option<Expression>, cause: Option<Expression> },
    Try { body: Vec<Statement>, handlers: Vec<ExceptHandler>,
        orelse: Vec<Statement>, finalbody: Vec<Statement> },
    Assert { test: Expression, msg: Option<Expression> },
    Import { names: Vec<Alias> },
    ImportFrom { module: Option<String>, names: Vec<Alias>, level: usize },
    Global { names: Vec<String> },
    Nonlocal { names: Vec<String> },
    Expr { value: Expression },
    Pass,
    Break,
    Continue,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    BoolOp { op: BoolOperator, values: Vec<Expression> },
    BinOp { left: Box<Expression>, op: Operator, right: Box<Expression> },
    UnaryOp { op: UnaryOperator, operand: Box<Expression> },
    Lambda { args: Box<Arguments>, body: Box<Expression> },
    If { test: Box<Expression>, body: Box<Expression>,
        orelse: Box<Expression> },
    Dict { keys: Vec<Expression>, values: Vec<Expression> },
    Set { elts: Vec<Expression> },
    ListComp { elt: Box<Expression>, generators: Vec<Comprehension> },
    SetComp { elt: Box<Expression>, generators: Vec<Comprehension> },
    DictComp { key: Box<Expression>, value: Box<Expression>,
        generators: Vec<Comprehension> },
    Generator { elt: Box<Expression>, generators: Vec<Comprehension> },
    None, // For DictComp/Arguments to have even length key/value lists
    Yield { value: Option<Box<Expression>> },
    YieldFrom { value: Box<Expression> },
    Compare { left: Box<Expression>, ops: Vec<CmpOperator>,
        comparators: Vec<Expression> },
    Call { func: Box<Expression>, args: Vec<Expression>,
        keywords: Vec<Keyword> },
    Num { n: Number },
    Str { s: String },
    NameConstant { value: Singleton },
    Ellipsis,
    Attribute { value: Box<Expression>, attr: String, ctx: ExprContext },
    Subscript { value: Box<Expression>, slice: Box<Slice>, ctx: ExprContext },
    Starred { value: Box<Expression>, ctx: ExprContext },
    Name { id: String, ctx: ExprContext },
    List { elts: Vec<Expression>, ctx: ExprContext },
    Tuple { elts: Vec<Expression>, ctx: ExprContext },
}

#[derive(Debug, PartialEq, Clone)]
pub enum ExprContext {
    Load,
    Store,
    Del,
    AugLoad,
    AugStore,
    Param
}

#[derive(Debug, PartialEq, Clone)]
pub enum Slice {
    Slice { lower: Option<Expression>, upper: Option<Expression>,
        step: Option<Expression> },
    ExtSlice { dims: Vec<Slice> },
    Index { value: Expression }
}

#[derive(Debug, PartialEq, Clone)]
pub enum BoolOperator {
    And,
    Or
}

#[derive(Debug, PartialEq, Clone)]
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

#[derive(Debug, PartialEq, Clone)]
pub enum UnaryOperator {
    Invert,
    Not,
    UAdd,
    USub
}

#[derive(Debug, PartialEq, Clone)]
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

#[derive(Debug, PartialEq, Clone)]
pub enum Comprehension {
    // No { ..., int: is_async } in Comprehension definition
    Comprehension { target: Expression, iter: Expression,
        ifs: Vec<Expression> }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ExceptHandler {
    ExceptHandler { etype: Option<Expression>, name: Option<String>,
        body: Vec<Statement> }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Arguments {
    Arguments { args: Vec<Arg>, vararg: Option<Arg>, kwonlyargs: Vec<Arg>,
        kw_defaults: Vec<Expression>, kwarg: Option<Arg>,
        defaults: Vec<Expression> }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Arg {
    Arg { arg: String, annotation: Option<Expression> }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Keyword {
    Keyword { arg: Option<String>, value: Expression }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Alias {
    Alias { name: String, asname: Option<String> }
}

#[derive(Debug, PartialEq, Clone)]
pub enum WithItem {
    WithItem { context_expr: Expression, optional_vars: Option<Expression> }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Singleton {
    None,
    True,
    False
}

#[derive(Debug, PartialEq, Clone)]
pub enum Number {
    DecInteger(String),
    BinInteger(String),
    OctInteger(String),
    HexInteger(String),
    Float(String),
    Imaginary(String)
}
