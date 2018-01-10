use std::error;
use std::fmt;

use ::lexer::tokens::Token;
use super::OptToken;

#[derive(Debug, PartialEq, Clone)]
pub enum ParserError
{
    InvalidSyntax(OptToken),
    UnexpectedToken(Token, OptToken),
    NonDefaultArgFollowsDefault(OptToken),
    PositionalArgAfterKeyword(OptToken),
    KeywordExpression(OptToken),
    UnexpectedEOF
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParserError::InvalidSyntax(ref opt) => {
                write!(f, "invalid syntax (line {})", opt.clone().unwrap().0)
            },
            ParserError::UnexpectedToken(ref expected, ref opt) => {
                write!(f, "unexpected token '{}', expected '{}' (line {})",
                    opt.clone().unwrap().1.unwrap().lexeme(),
                    expected.clone().lexeme(),
                    opt.clone().unwrap().0)
            },
            ParserError::NonDefaultArgFollowsDefault(ref opt) => {
                write!(f, "non-default argument follows default \
                    argument (line {})", opt.clone().unwrap().0)
            },
            ParserError::PositionalArgAfterKeyword(ref opt) => {
                write!(f, "positional argument follows keyword \
                    argument unpacking (line {})", opt.clone().unwrap().0)
            },
            ParserError::KeywordExpression(ref opt) => {
                write!(f, "keyword can't be expression (line {})",
                    opt.clone().unwrap().0)
            },
            ParserError::UnexpectedEOF => write!(f, "unexpected EOF")
        }
    }
}

impl error::Error for ParserError {
    fn description(&self) -> &str {
        match *self {
            ParserError::InvalidSyntax(_) => "invalid syntax",
            ParserError::UnexpectedToken(..) => "unexpected token",
            ParserError::NonDefaultArgFollowsDefault(_) =>
                "non-default argument follows default argument",
            ParserError::PositionalArgAfterKeyword(_) =>
                "positional argument follows keywork argument unpacking",
            ParserError::KeywordExpression(_) => "keyword can't be expression",
            ParserError::UnexpectedEOF => "unexpected EOF"
        }
    }
}
