use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Str(String),
    Num(Number),
    Object
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Type::Str(ref s) => write!(f, "{}", s),
            Type::Num(ref n) => write!(f, "{}", n),
            Type::Object => write!(f, "object")
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Number {
    Integer(i64),
    Float(f64),
    Imaginary(String)
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Number::Integer(ref val) =>
                write!(f, "i32 {}", val),
            Number::Float(ref val) =>
                write!(f, "float {:.32}", val),
            Number::Imaginary(ref val) =>
                write!(f, "{}", val),
        }
    }
}
