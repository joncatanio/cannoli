use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Str(String),
    Num { n: Number },
    Object
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Type::Str(ref s) => write!(f, "{}", s),
            Type::Num { ref n } => write!(f, "{}", n),
            Type::Object => write!(f, "object")
        }
    }
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

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Number::DecInteger(ref s) =>
                write!(f, "i32 {}", s),
            Number::BinInteger(ref s) =>
                write!(f, "{}", s),
            Number::OctInteger(ref s) =>
                write!(f, "{}", s),
            Number::HexInteger(ref s) =>
                write!(f, "{}", s),
            Number::Float(ref s) =>
                write!(f, "{}", s),
            Number::Imaginary(ref s) =>
                write!(f, "{}", s),
        }
    }
}
