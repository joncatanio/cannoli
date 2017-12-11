use std::error;
use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum LexerError
{
   BadLineContinuation,
   UnterminatedTripleString,
   UnterminatedString,
   InvalidCharacter(char),
   Dedent,
   HexEscapeShort,
   MalformedUnicodeEscape,
   MalformedNamedUnicodeEscape,
   UnknownUnicodeName(String),
   BytesNonASCII,
   MissingDigits,
   MalformedFloat,
   MalformedImaginary,
   InvalidSymbol(String),
   Internal(String),
}

impl fmt::Display for LexerError
{
   fn fmt(&self, f: &mut fmt::Formatter)
      -> fmt::Result
   {
      match *self
      {
         LexerError::BadLineContinuation =>
            write!(f, "bad line continuation"),
         LexerError::UnterminatedTripleString =>
            write!(f, "unterminated triple-quoted string"),
         LexerError::UnterminatedString =>
            write!(f, "unterminated string"),
         LexerError::InvalidCharacter(ref c) =>
            write!(f, "invalid character '{}'", c),
         LexerError::Dedent =>
            write!(f, "misaligned dedent"),
         LexerError::HexEscapeShort =>
            write!(f, "missing digits in hex escape"),
         LexerError::MalformedUnicodeEscape =>
            write!(f, "malformed unicode escape"),
         LexerError::MalformedNamedUnicodeEscape =>
            write!(f, "malformed named unicode escape"),
         LexerError::UnknownUnicodeName(ref s) =>
            write!(f, "unknown unicode name '{}'", s),
         LexerError::MissingDigits =>
            write!(f, "missing digits"),
         LexerError::BytesNonASCII =>
            write!(f, "bytes cannot contain non-ASCII characters"),
         LexerError::MalformedFloat =>
            write!(f, "malformed floating point number"),
         LexerError::MalformedImaginary =>
            write!(f, "malformed imaginary number"),
         LexerError::InvalidSymbol(ref s) =>
            write!(f, "invalid symbol '{}'", s),
         LexerError::Internal(ref s) =>
            write!(f, "internal error: {}", s),
      }
   }
}

impl error::Error for LexerError
{
   fn description(&self)
      -> &str
   {
      match *self
      {
         LexerError::BadLineContinuation => "bad line continuation",
         LexerError::UnterminatedTripleString =>
            "unterminated triple-quoted string",
         LexerError::UnterminatedString => "unterminated string",
         LexerError::InvalidCharacter(_) => "invalid character",
         LexerError::Dedent => "misaligned dedent",
         LexerError::HexEscapeShort => "missing digits in hex escape",
         LexerError::MalformedUnicodeEscape => "malformed unicode escape",
         LexerError::MalformedNamedUnicodeEscape =>
            "malformed named unicode escape",
         LexerError::UnknownUnicodeName(_) => "unknown unicode name",
         LexerError::BytesNonASCII =>
            "bytes cannot contain non-ASCII characters",
         LexerError::MissingDigits => "missing digits",
         LexerError::MalformedFloat => "malformed floating point number",
         LexerError::MalformedImaginary => "malformed imaginary number",
         LexerError::InvalidSymbol(_) => "invalid symbol",
         LexerError::Internal(_) => "internal error",
      }
   }
}
