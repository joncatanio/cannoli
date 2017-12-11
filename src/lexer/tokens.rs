use std::collections::HashMap;
use lexer::errors::LexerError;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Token
{
   Newline,
   Indent,
   Dedent,
   False,
   None,
   True,
   And,
   As,
   Assert,
   //Async,
   //Await,
   Break,
   Class,
   Continue,
   Def,
   Del,
   Elif,
   Else,
   Except,
   Finally,
   For,
   From,
   Global,
   If,
   Import,
   In,
   Is,
   Lambda,
   Nonlocal,
   Not,
   Or,
   Pass,
   Raise,
   Return,
   Try,
   While,
   With,
   Yield,
   Plus,
   Minus,
   Times,
   Exponent,
   Divide,
   DivideFloor,
   Mod,
   At,
   Lshift,
   Rshift,
   BitAnd,
   BitOr,
   BitXor,
   BitNot,
   LT,
   GT,
   LE,
   GE,
   EQ,
   NE,
   Lparen,
   Rparen,
   Lbracket,
   Rbracket,
   Lbrace,
   Rbrace,
   Comma,
   Colon,
   Dot,
   Ellipsis,
   Semi,
   Arrow,
   Assign,
   AssignPlus,
   AssignMinus,
   AssignTimes,
   AssignDivide,
   AssignDivideFloor,
   AssignMod,
   AssignAt,
   AssignBitAnd,
   AssignBitOr,
   AssignBitXor,
   AssignRshift,
   AssignLshift,
   AssignExponent,
   Quote,
   DoubleQuote,
   Identifier(String),
   String(String),
   Bytes(Vec<u8>),
   DecInteger(String),
   BinInteger(String),
   OctInteger(String),
   HexInteger(String),
   Float(String),
   Imaginary(String),
}

impl Token
{
   pub fn is_decimal_integer(&self)
      -> bool
   {
      match self
      {
         &Token::DecInteger(_) => true,
         _ => false,
      }
   }

   pub fn is_float(&self)
      -> bool
   {
      match self
      {
         &Token::Float(_) => true,
         _ => false,
      }
   }

   pub fn lexeme(self)
      -> String
   {
      match self
      {
         Token::Identifier(s) | Token::String(s) |
            Token::DecInteger(s) | Token::BinInteger(s) |
            Token::OctInteger(s) | Token::HexInteger(s) |
            Token::Float(s) | Token::Imaginary(s) => s,
         Token::Bytes(s) => String::from_utf8(s).unwrap(),
         _ =>
         {
            match LEXEMES.get(&self)
            {
               Some(&s) => s.to_owned(),
               None => unreachable!(),
            }
         }
      }
   }

   pub fn with_equal(&self)
      -> Self
   {
      match self
      {
         &Token::Plus => Token::AssignPlus,
         &Token::Minus => Token::AssignMinus,
         &Token::Times => Token::AssignTimes,
         &Token::Exponent => Token::AssignExponent,
         &Token::Divide => Token::AssignDivide,
         &Token::DivideFloor => Token::AssignDivideFloor,
         &Token::BitAnd => Token::AssignBitAnd,
         &Token::BitOr => Token::AssignBitOr,
         &Token::BitXor => Token::AssignBitXor,
         &Token::Mod => Token::AssignMod,
         &Token::At => Token::AssignAt,
         &Token::Assign => Token::EQ,
         &Token::LT => Token::LE,
         &Token::Lshift => Token::AssignLshift,
         &Token::GT => Token::GE,
         &Token::Rshift => Token::AssignRshift,
         _ => self.clone()
      }
   }
}

pub fn keyword_lookup(token_str: &str)
   -> Token
{
   match KEYWORDS.get(token_str)
   {
      Some(token) => token.clone(),
      None => Token::Identifier(token_str.to_owned()),
   }
}

pub fn symbol_lookup(token_str: &str)
   -> Result<Token, LexerError>
{
   match SYMBOLS.get(token_str)
   {
      Some(token) => Ok(token.clone()),
      None => Err(LexerError::InvalidSymbol(token_str.to_owned())),
   }
}

lazy_static!
{
   static ref KEYWORDS : HashMap<&'static str, Token> = initialize_keywords();
   static ref LEXEMES : HashMap<Token, &'static str> = initialize_lexemes();
   static ref SYMBOLS : HashMap<&'static str, Token> = initialize_symbols();
}

fn initialize_keywords()
   -> HashMap<&'static str, Token>
{
   let mut keywords = HashMap::new();

   keywords.insert("False", Token::False);
   keywords.insert("None", Token::None);
   keywords.insert("True", Token::True);
   keywords.insert("and", Token::And);
   keywords.insert("as", Token::As);
   keywords.insert("assert", Token::Assert);
   //keywords.insert("async", Token::Async);
   //keywords.insert("await", Token::Await);
   keywords.insert("break", Token::Break);
   keywords.insert("class", Token::Class);
   keywords.insert("continue", Token::Continue);
   keywords.insert("def", Token::Def);
   keywords.insert("del", Token::Del);
   keywords.insert("elif", Token::Elif);
   keywords.insert("else", Token::Else);
   keywords.insert("except", Token::Except);
   keywords.insert("finally", Token::Finally);
   keywords.insert("for", Token::For);
   keywords.insert("from", Token::From);
   keywords.insert("global", Token::Global);
   keywords.insert("if", Token::If);
   keywords.insert("import", Token::Import);
   keywords.insert("in", Token::In);
   keywords.insert("is", Token::Is);
   keywords.insert("lambda", Token::Lambda);
   keywords.insert("nonlocal", Token::Nonlocal);
   keywords.insert("not", Token::Not);
   keywords.insert("or", Token::Or);
   keywords.insert("pass", Token::Pass);
   keywords.insert("raise", Token::Raise);
   keywords.insert("return", Token::Return);
   keywords.insert("try", Token::Try);
   keywords.insert("while", Token::While);
   keywords.insert("with", Token::With);
   keywords.insert("yield", Token::Yield);

   keywords
}

fn initialize_lexemes()
   -> HashMap<Token, &'static str>
{
   let mut lexemes = HashMap::new();

   lexemes.insert(Token::Newline, "\n");
   lexemes.insert(Token::Indent, "INDENT");
   lexemes.insert(Token::Dedent, "DEDENT");
   lexemes.insert(Token::False, "False");
   lexemes.insert(Token::None, "None");
   lexemes.insert(Token::True, "True");
   lexemes.insert(Token::And, "and");
   lexemes.insert(Token::As, "as");
   lexemes.insert(Token::Assert, "assert");
   //lexemes.insert(Token::Async, "async");
   //lexemes.insert(Token::Await, "await");
   lexemes.insert(Token::Break, "break");
   lexemes.insert(Token::Class, "class");
   lexemes.insert(Token::Continue, "continue");
   lexemes.insert(Token::Def, "def");
   lexemes.insert(Token::Del, "del");
   lexemes.insert(Token::Elif, "elif");
   lexemes.insert(Token::Else, "else");
   lexemes.insert(Token::Except, "except");
   lexemes.insert(Token::Finally, "finally");
   lexemes.insert(Token::For, "for");
   lexemes.insert(Token::From, "from");
   lexemes.insert(Token::Global, "global");
   lexemes.insert(Token::If, "if");
   lexemes.insert(Token::Import, "import");
   lexemes.insert(Token::In, "in");
   lexemes.insert(Token::Is, "is");
   lexemes.insert(Token::Lambda, "lambda");
   lexemes.insert(Token::Nonlocal, "nonlocal");
   lexemes.insert(Token::Not, "not");
   lexemes.insert(Token::Or, "or");
   lexemes.insert(Token::Pass, "pass");
   lexemes.insert(Token::Raise, "raise");
   lexemes.insert(Token::Return, "return");
   lexemes.insert(Token::Try, "try");
   lexemes.insert(Token::While, "while");
   lexemes.insert(Token::With, "with");
   lexemes.insert(Token::Yield, "yield");
   lexemes.insert(Token::Plus, "+");
   lexemes.insert(Token::Minus, "-");
   lexemes.insert(Token::Times, "*");
   lexemes.insert(Token::Exponent, "**");
   lexemes.insert(Token::Divide, "/");
   lexemes.insert(Token::DivideFloor, "//");
   lexemes.insert(Token::Mod, "%");
   lexemes.insert(Token::At, "@");
   lexemes.insert(Token::Lshift, "<<");
   lexemes.insert(Token::Rshift, ">>");
   lexemes.insert(Token::BitAnd, "&");
   lexemes.insert(Token::BitOr, "|");
   lexemes.insert(Token::BitXor, "^");
   lexemes.insert(Token::BitNot, "~");
   lexemes.insert(Token::LT, "<");
   lexemes.insert(Token::GT, ">");
   lexemes.insert(Token::LE, "<=");
   lexemes.insert(Token::GE, ">=");
   lexemes.insert(Token::EQ, "==");
   lexemes.insert(Token::NE, "!=");
   lexemes.insert(Token::Lparen, "(");
   lexemes.insert(Token::Rparen, ")");
   lexemes.insert(Token::Lbracket, "[");
   lexemes.insert(Token::Rbracket, "]");
   lexemes.insert(Token::Lbrace, "{");
   lexemes.insert(Token::Rbrace, "}");
   lexemes.insert(Token::Comma, ",");
   lexemes.insert(Token::Colon, ":");
   lexemes.insert(Token::Dot, ".");
   lexemes.insert(Token::Ellipsis, "...");
   lexemes.insert(Token::Semi, ";");
   lexemes.insert(Token::Arrow, "->");
   lexemes.insert(Token::Assign, "=");
   lexemes.insert(Token::AssignPlus, "+=");
   lexemes.insert(Token::AssignMinus, "-=");
   lexemes.insert(Token::AssignTimes, "*=");
   lexemes.insert(Token::AssignDivide, "/=");
   lexemes.insert(Token::AssignDivideFloor, "//=");
   lexemes.insert(Token::AssignMod, "%");
   lexemes.insert(Token::AssignAt, "@=");
   lexemes.insert(Token::AssignBitAnd, "&=");
   lexemes.insert(Token::AssignBitOr, "|=");
   lexemes.insert(Token::AssignBitXor, "^=");
   lexemes.insert(Token::AssignRshift, ">>=");
   lexemes.insert(Token::AssignLshift, "<<=");
   lexemes.insert(Token::AssignExponent, "**=");
   lexemes.insert(Token::Quote, "'");
   lexemes.insert(Token::DoubleQuote, "\"");

   lexemes
}

fn initialize_symbols()
   -> HashMap<&'static str, Token>
{
   let mut symbols = HashMap::new();

   symbols.insert("+", Token::Plus);
   symbols.insert("-", Token::Minus);
   symbols.insert("*", Token::Times);
   symbols.insert("**", Token::Exponent);
   symbols.insert("/", Token::Divide);
   symbols.insert("//", Token::DivideFloor);
   symbols.insert("%", Token::Mod);
   symbols.insert("@", Token::At);
   symbols.insert("<<", Token::Lshift);
   symbols.insert(">>", Token::Rshift);
   symbols.insert("&", Token::BitAnd);
   symbols.insert("|", Token::BitOr);
   symbols.insert("^", Token::BitXor);
   symbols.insert("~", Token::BitNot);
   symbols.insert("<", Token::LT);
   symbols.insert(">", Token::GT);
   symbols.insert("<=", Token::LE);
   symbols.insert(">=", Token::GE);
   symbols.insert("==", Token::EQ);
   symbols.insert("!=", Token::NE);
   symbols.insert("(", Token::Lparen);
   symbols.insert(")", Token::Rparen);
   symbols.insert("[", Token::Lbracket);
   symbols.insert("]", Token::Rbracket);
   symbols.insert("{", Token::Lbrace);
   symbols.insert("}", Token::Rbrace);
   symbols.insert(",", Token::Comma);
   symbols.insert(":", Token::Colon);
   symbols.insert(".", Token::Dot);
   symbols.insert("...", Token::Ellipsis);
   symbols.insert(";", Token::Semi);
   symbols.insert("->", Token::Arrow);
   symbols.insert("=", Token::Assign);
   symbols.insert("+=", Token::AssignPlus);
   symbols.insert("-=", Token::AssignMinus);
   symbols.insert("*=", Token::AssignTimes);
   symbols.insert("/=", Token::AssignDivide);
   symbols.insert("//=", Token::AssignDivideFloor);
   symbols.insert("%=", Token::AssignMod);
   symbols.insert("@=", Token::AssignAt);
   symbols.insert("&=", Token::AssignBitAnd);
   symbols.insert("|=", Token::AssignBitOr);
   symbols.insert("^=", Token::AssignBitXor);
   symbols.insert(">>=", Token::AssignRshift);
   symbols.insert("<<=", Token::AssignLshift);
   symbols.insert("**=", Token::AssignExponent);

   symbols
}
