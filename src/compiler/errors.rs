use std::error;
use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum CompilerError
{
    IOError(String),
    ParserError(String),
    NameError(String)
}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CompilerError::IOError(ref s) => write!(f, "{}", s),
            CompilerError::ParserError(ref s) =>
                write!(f, "ParserError: {}", s),
            CompilerError::NameError(ref s) =>
                write!(f, "NameError: name '{}' is not defined", s)
        }
    }
}

impl error::Error for CompilerError {
    fn description(&self) -> &str {
        match *self {
            CompilerError::IOError(_) => "i/o error",
            CompilerError::ParserError(_) => "parser error",
            CompilerError::NameError(_) => "name error"
        }
    }
}
