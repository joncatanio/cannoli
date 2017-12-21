pub mod errors;
pub mod iter;
pub mod tokens;

/// It should be noted that indentation checks do not verify that mixed
/// spaces and tabs do not depend on the size of a tab stop for correctness.

use regex::{Regex, Captures, FindCaptures};
use std::char;
use std::cmp;
use std::iter::Peekable;
use unicode_names;

use self::tokens::{Token, keyword_lookup, symbol_lookup};
use self::errors::LexerError;


const TAB_STOP_SIZE: u32 = 8;

pub type ResultToken = Result<Token, LexerError>;

pub struct Lexer<'a>
{
   lexer: Peekable<StringJoiningLexer<'a>>
}

impl <'a> Lexer<'a>
{
   pub fn new(input: &str)
      -> Lexer
   {
      Lexer{lexer:
         StringJoiningLexer::new(
            BytesJoiningLexer::new(
               InternalLexer::new(input)
            )
         ).peekable()}
   }
}

impl <'a> Iterator for Lexer<'a>
{
   type Item = (usize, ResultToken);

   fn next(&mut self)
      -> Option<Self::Item>
   {
      self.lexer.next()
   }
}

pub struct StringJoiningLexer<'a>
{
   lexer: Peekable<BytesJoiningLexer<'a>>
}

impl <'a> StringJoiningLexer<'a>
{
   pub fn new<'b>(lexer: BytesJoiningLexer<'b>)
      -> StringJoiningLexer<'b>
   {
      StringJoiningLexer{lexer: lexer.peekable()}
   }

   fn string_follows(&mut self)
      -> Option<String>
   {
      match self.lexer.peek()
      {
         Some(&(_, Ok(Token::String(_)))) =>
         {
            Some(self.lexer.next().unwrap().1.unwrap().lexeme())
         },
         _ => None,
      }
   }
}

impl <'a> Iterator for StringJoiningLexer<'a>
{
   type Item = (usize, ResultToken);

   fn next(&mut self)
      -> Option<Self::Item>
   {
      match self.lexer.next()
      {
         Some((line_number, Ok(Token::String(s)))) =>
         {
            let mut token_str = s.clone();
            while let Some(follow) = self.string_follows()
            {
               token_str.push_str(&follow)
            }
            Some((line_number, Ok(Token::String(token_str))))
         },
         result => result,
      }
   }
}

pub struct BytesJoiningLexer<'a>
{
   lexer: Peekable<InternalLexer<'a>>
}

impl <'a> BytesJoiningLexer<'a>
{
   pub fn new<'b>(lexer: InternalLexer<'b>)
      -> BytesJoiningLexer<'b>
   {
      BytesJoiningLexer{lexer: lexer.peekable()}
   }

   fn bytes_follows(&mut self)
      -> Option<Vec<u8>>
   {
      match self.lexer.peek()
      {
         Some(&(_, Ok(Token::Bytes(_)))) =>
         {
            match self.lexer.next().unwrap().1.unwrap()
            {
               Token::Bytes(bytes) => Some(bytes),
               _ => unreachable!(),
            }
         },
         _ => None,
      }
   }
}

impl <'a> Iterator for BytesJoiningLexer<'a>
{
   type Item = (usize, ResultToken);

   fn next(&mut self)
      -> Option<Self::Item>
   {
      match self.lexer.next()
      {
         Some((line_number, Ok(Token::Bytes(s)))) =>
         {
            let mut token_vec = s.clone();
            while let Some(mut follow) = self.bytes_follows()
            {
               token_vec.append(&mut follow)
            }
            Some((line_number, Ok(Token::Bytes(token_vec))))
         },
         result => result,
      }
   }
}

pub struct InternalLexer<'a>
{
   indent_stack: Vec<u32>,
   dedent_count: i32,            // negative value to indicate a misalignment
   open_braces: u32,
   text: &'a str,
   line_start: bool,
   line_number: usize,
}

impl <'a> Iterator for InternalLexer<'a>
{
   type Item = (usize, ResultToken);

   fn next(&mut self)
      -> Option<Self::Item>
   {
      self.next_token()
   }
}

impl <'a> InternalLexer<'a>
{
   pub fn new(input: &str)
      -> InternalLexer
   {
      InternalLexer{indent_stack: vec![0],
         dedent_count: 0,
         text: input,
         line_number: 1,
         line_start: true,
         open_braces: 0,
      }
   }

   fn update_text(&mut self, end: usize)
   {
      self.text = &self.text[end..];
   }

   fn next_token(&mut self)
      -> Option<(usize, ResultToken)>
   {
      if self.text.len() > 0
      {
         if self.line_start
         {
            self.process_line_start()
         }
         else if self.dedent_count != 0
         {
            Some(self.process_dedents())
         }
         else
         {
            consume_space_to_next(&mut self.text);
            if let Some((_, end)) = LOGICAL_EOL_RE.find(self.text)
            {
               self.process_end_of_line(end)
            }
            else if STRING_START_RE.is_match(self.text)
            {
               Some(self.process_string())
            }
            else if BYTES_START_RE.is_match(self.text)
            {
               Some(self.process_byte_string())
            }
            else if let Some((_, end)) = ID_RE.find(self.text)
            {
               Some(self.process_identifier(end))
            }
            else if let Some((_, end)) = FLOAT_RE.find(self.text)
            {
               Some(self.process_float(end))
            }
            else if let Some((_, end)) = INT_IMG_RE.find(self.text)
            {
               Some(self.process_number(end, |s| Token::Imaginary(s)))
            }
            else if let Some((_, end)) = INVALID_DEC_RE.find(self.text)
            {
               self.update_text(end);
               Some((self.line_number, Err(LexerError::MalformedFloat)))
            }
            else if let Some((_, end)) = HEX_RE.find(self.text)
            {
               Some(self.process_number(end, |s| Token::HexInteger(s)))
            }
            else if let Some((_, end)) = OCT_RE.find(self.text)
            {
               Some(self.process_number(end, |s| Token::OctInteger(s)))
            }
            else if let Some((_, end)) = BIN_RE.find(self.text)
            {
               Some(self.process_number(end, |s| Token::BinInteger(s)))
            }
            else if let Some((_, end)) = INVALID_ZERO_PRE_RE.find(self.text)
            {
               self.update_text(end);
               Some((self.line_number, Err(LexerError::MissingDigits)))
            }
            else if let Some((_, end)) = DEC_RE.find(self.text)
            {
               Some(self.process_number(end, |s| Token::DecInteger(s)))
            }
            else if let Some((_, end)) = LINE_JOIN_START_RE.find(self.text)
            {
               self.process_line_join(end)
            }
            else
            {
               Some(self.process_symbol())
            }
         }
      }
      else if self.indent_stack.len() > 1
      {
         self.indent_stack.pop();
         Some((0, Ok(Token::Dedent)))
      }
      else
      {
         None
      }
   }

   fn process_line_join(&mut self, end: usize)
      -> Option<(usize, ResultToken)>
   {
      if let Some((_, end)) = LINE_JOIN_RE.find(self.text)
      {
         // explicit line join
         self.line_number += 1;
         self.update_text(end);
         self.next_token()
      }
      else
      {
         self.update_text(end);
         Some((self.line_number, Err(LexerError::BadLineContinuation)))
      }
   }

   fn process_string(&mut self)
      -> (usize, ResultToken)
   {
      let (_, end) = STRING_PREFIX_RE.find(self.text).unwrap();
      let caps = STRING_PREFIX_RE.captures(self.text).unwrap();
      let raw = caps.at(1).is_some();
      let quote = caps.at(2).unwrap();

      self.update_text(end);

      let (re, fail, err) = determine_string_processing(quote);

      match re.find(self.text)
      {
         Some((_, end)) =>
         {
            self.build_string_contents(end, re, raw)
         },
         None =>
         {
            self.handle_string_err(fail, err)
         },
      }
   }

   fn build_string_contents(&mut self, end: usize, re: &Regex, raw: bool)
      -> (usize, ResultToken)
   {
      let caps = re.captures(self.text).unwrap();
      let contents = caps.at(1).unwrap_or("");
      let newlines = NEWLINE_RE.find_iter(&contents).count();

      let current_line_number = self.line_number;
      self.update_text(end);
      self.line_number += newlines;

      let expanded =
         if !raw
         {
            if let Some(err) = check_escape_errors(contents)
            {
               return (current_line_number, Err(err))
            }
            // check_escape_errors also iterates over structurally valid
            // named unicode characters - duplicating some of
            // the iteration done below in replace_all - this is
            // kept separate for code clarity, but could be merged
            ESCAPES_RE.replace_all(contents, |caps: &Captures|
               process_escape_sequence(caps.at(1).unwrap_or("")))
         }
         else
         {
            contents.to_owned()
         };
      (current_line_number, Ok(Token::String(expanded)))
   }

   fn build_bytes_contents(&mut self, end: usize, re: &Regex, raw: bool)
      -> (usize, ResultToken)
   {
      let caps = re.captures(self.text).unwrap();
      let contents = caps.at(1).unwrap_or("");
      let newlines = NEWLINE_RE.find_iter(&contents).count();

      let current_line_number = self.line_number;
      self.update_text(end);
      self.line_number += newlines;

      if NON_ASCII_RE.is_match(contents)
      {
         return (current_line_number, Err(LexerError::BytesNonASCII));
      }

      let expanded =
         if !raw
         {
            if let Some(err) = check_byte_escape_errors(contents)
            {
               return (current_line_number, Err(err))
            }
            replace_string_bytes(&ESCAPES_BYTES_RE, contents, |caps: &Captures|
               process_byte_escape_sequence(caps.at(1).unwrap_or("")))
         }
         else
         {
            contents.as_bytes().to_vec()
         };
      (current_line_number, Ok(Token::Bytes(expanded)))
   }

   fn handle_string_err(&mut self, fail: &Regex, err: LexerError)
      -> (usize, ResultToken)
   {
      let (_, end) = fail.find(self.text).unwrap();
      let newlines = NEWLINE_RE.find_iter(&self.text[..end]).count();
      self.update_text(end);
      self.line_number += newlines;
      (self.line_number, Err(err))
   }

   fn process_byte_string(&mut self)
      -> (usize, ResultToken)
   {
      let (_, end) = BYTES_PREFIX_RE.find(self.text).unwrap();
      let caps = BYTES_PREFIX_RE.captures(self.text).unwrap();
      let raw = caps.at(1).is_some() || caps.at(2).is_some();
      let quote = caps.at(3).unwrap();

      println!("is raw: {} '{}' '{}'", raw, caps.at(1).unwrap_or(""),
         caps.at(2).unwrap_or(""));
      self.update_text(end);

      let (re, fail, err) = determine_string_processing(quote);

      match re.find(self.text)
      {
         Some((_, end)) =>
         {
            self.build_bytes_contents(end, re, raw)
         },
         None =>
         {
            self.handle_string_err(fail, err)
         },
      }
   }


   fn process_line_start(&mut self)
      -> Option<(usize, ResultToken)>
   {
      let indentation = count_indentation(&mut self.text);
      self.line_start = false;  // next attempt processes line as normal
      if let Some(&previous_indent) = self.indent_stack.last()
      {
         if let Some((_, end)) = LOGICAL_EOL_RE.find(self.text)
         {
            // logically blank line, ignore entirely
            self.update_text(end);
            self.line_number += 1;
            self.line_start = true;
            self.next_token()
         }
         else if indentation > previous_indent
         {
            self.indent_stack.push(indentation);
            Some((self.line_number, Ok(Token::Indent)))
         }
         else if indentation < previous_indent
         {
            let stack_len = self.indent_stack.len();
            let mut i = stack_len - 1;
            while indentation < self.indent_stack[i]
            {
               i -= 1;
            }
            self.indent_stack.truncate(i + 1);
            self.dedent_count = (stack_len - 1 - i) as i32;
            if self.indent_stack[i] != indentation
            {
               self.dedent_count = -self.dedent_count; // negate to flag error
            }
            self.next_token()
         }
         else
         {
            // same indentation level, just get token
            self.next_token()
         }
      }
      else
      {
         panic!("Internal indentation stack error!")
      }
   }

   fn process_dedents(&mut self)
      -> (usize, ResultToken)
   {
      if self.dedent_count == -1
      {
         self.dedent_count = 0;
         (self.line_number, Err(LexerError::Dedent))
      }
      else
      {
         self.dedent_count += if self.dedent_count < 0 {1} else {-1};
         (self.line_number, Ok(Token::Dedent))
      }
   }

   fn process_symbol(&mut self)
      -> (usize, ResultToken)
   {
      if let Some((_, end)) = SYMBOLS_RE.find(self.text)
      {
         let result = &self.text[..end];
         self.update_text(end);

         match result
         {
            "(" | "[" | "{" =>
            {
               self.open_braces += 1;
               (self.line_number, symbol_lookup(result))
            },
            ")" | "]" | "}" =>
            {
               self.open_braces = cmp::max(0, self.open_braces - 1);
               (self.line_number, symbol_lookup(result))
            },
            sym => (self.line_number, symbol_lookup(sym))
         }
      }
      else
      {
         let c = &self.text[..1];
         self.update_text(1); // skip one to allow progress
         (self.line_number, Err(LexerError::InvalidSymbol(c.to_owned())))
      }
   }

   fn process_end_of_line(&mut self, end: usize)
      -> Option<(usize, ResultToken)>
   {
      self.update_text(end);
      let current_line_number = self.line_number;
      self.line_number += 1;
      if self.open_braces == 0
      {
         self.line_start = true;
         Some((current_line_number, Ok(Token::Newline)))
      }
      else
      {
         // implicit join, so the newline does not produce a token and
         // the "start" of the next line should be ignored -- i.e., no
         // indentation processing
         self.next_token()
      }
   }

   fn process_identifier(&mut self, end: usize)
      -> (usize, ResultToken)
   {
      let token = keyword_lookup(&self.text[0..end]);
      self.update_text(end);
      (self.line_number, Ok(token))
   }

   fn process_float(&mut self, end: usize)
      -> (usize, ResultToken)
   {
      let rest = &self.text[end..];
      if let Some((_, end_img)) = IMG_SUFFIX_RE.find(rest)
      {
         let token_str = self.text[..(end + end_img)].to_owned();
         self.update_text(end + end_img);
         (self.line_number, Ok(Token::Imaginary(token_str)))
      }
      else
      {
         let token_str = self.text[..end].to_owned();
         self.update_text(end);
         (self.line_number, Ok(Token::Float(token_str)))
      }
   }

   fn process_number<F>(&mut self, end: usize, ctor: F)
      -> (usize, ResultToken)
      where F : Fn(String) -> Token
   {
      let token_str = self.text[0..end].to_owned();
      self.update_text(end);
      (self.line_number, Ok(ctor(token_str)))
   }
}

fn replace_string_bytes<F>(re: &Regex, contents: &str, process: F)
   -> Vec<u8>
   where F: Fn(&Captures) -> Vec<u8>
{
   let mut bytes = vec![];
   let mut pos = 0;

   for (start, end) in re.find_iter(contents)
   {
      push_all(&mut bytes, contents[pos..start].as_bytes());
      if let Some(caps) = re.captures(&contents[start..end])
      {
         bytes.append(&mut process(&caps));
      }
      pos = end;
   }
   push_all(&mut bytes, contents[pos..].as_bytes());

   bytes
}

fn determine_string_processing(quote: &str)
   -> (&Regex, &Regex, LexerError)
{
   match quote
   {
      "'" => (&*STRING_SINGLE_QUOTE_RE, &*STRING_FAIL_RE,
               LexerError::UnterminatedString),
      "'''" => (&*STRING_TRIPLE_SINGLE_QUOTE_RE, &*STRING_TRIPLE_FAIL_RE,
               LexerError::UnterminatedTripleString),
      "\"" => (&*STRING_DOUBLE_QUOTE_RE, &*STRING_FAIL_RE,
               LexerError::UnterminatedString),
      "\"\"\"" => (&*STRING_TRIPLE_DOUBLE_QUOTE_RE, &*STRING_TRIPLE_FAIL_RE,
               LexerError::UnterminatedTripleString),
      _ => unreachable!(),
   }
}

fn process_escape_sequence(escaped: &str)
   -> String
{
   match escaped
   {
      "\n" | "\r" | "\r\n" => "".to_owned(),
      "\\" => "\\".to_owned(),
      "'" => "'".to_owned(),
      "\"" => "\"".to_owned(),
      "a" => "\x07".to_owned(),
      "b" => "\x08".to_owned(),
      "f" => "\x0C".to_owned(),
      "n" => "\n".to_owned(),
      "r" => "\r".to_owned(),
      "t" => "\t".to_owned(),
      "v" => "\x0B".to_owned(),
      escaped =>
      {
         if OCT_ESCAPE_RE.is_match(escaped)
         {
            char::from_u32(u32::from_str_radix(escaped, 8)
               .unwrap()).unwrap().to_string()
         }
         else if HEX_ESCAPE_RE.is_match(escaped)
         {
            char::from_u32(u32::from_str_radix(&escaped[1..], 16)
               .unwrap()).unwrap().to_string()
         }
         else if UNICODE_ESCAPE_RE.is_match(escaped)
         {
            char::from_u32(u32::from_str_radix(&escaped[1..], 16)
               .unwrap()).unwrap().to_string()
         }
         else if let Some(name_cap) = UNICODE_NAME_RE.captures(escaped)
         {
            match unicode_names::character(name_cap.at(1).unwrap_or(""))
            {
               Some(c) => c.to_string(),
               _ => "\\".to_owned() + escaped,
            }
         }
         else
         {
            "\\".to_owned() + escaped
         }
      },
   }
}

fn check_escape_errors(s: &str)
   -> Option<LexerError>
{
   check_non_named_escape_errors(ESCAPES_FAIL_RE.captures(s))
      .or_else(|| check_named_escape_errors(
         UNICODE_NAMED_ESCAPE_RE.captures_iter(s)))
}

fn check_non_named_escape_errors(caps: Option<Captures>)
   -> Option<LexerError>
{
   if caps.is_none() { return None; }
   let caps = caps.unwrap();

   if let Some(_) = caps.name("badu")
   {
      Some(LexerError::MalformedUnicodeEscape)
   }
   else if let Some(_) = caps.name("badU")
   {
      Some(LexerError::MalformedUnicodeEscape)
   }
   else if let Some(_) = caps.name("end")
   {
      Some(LexerError::MalformedNamedUnicodeEscape)
   }
   else if let Some(_) = caps.name("start")
   {
      Some(LexerError::MalformedNamedUnicodeEscape)
   }
   else if let Some(_) = caps.name("badx")
   {
      Some(LexerError::HexEscapeShort)
   }
   else
   {
      None
   }
}

fn check_named_escape_errors(caps: FindCaptures)
   -> Option<LexerError>
{
   for cap in caps
   {
      let cap_name = cap.at(1).unwrap_or("");
      match unicode_names::character(cap_name)
      {
         Some(_) => (),
         _ => return Some(LexerError::UnknownUnicodeName(cap_name.to_owned())),
      }
   }
   None
}

fn check_byte_escape_errors(s: &str)
   -> Option<LexerError>
{
   let caps = ESCAPES_BYTES_FAIL_RE.captures(s);
   if caps.is_none() { return None; }
   let caps = caps.unwrap();

   if let Some(_) = caps.name("badx")
   {
      Some(LexerError::HexEscapeShort)
   }
   else
   {
      None
   }
}

fn process_byte_escape_sequence(escaped: &str)
   -> Vec<u8>
{
   match escaped
   {
      "\n" | "\r" | "\r\n" => vec![],
      "\\" => vec!['\\' as u8],
      "'" => vec!['\'' as u8],
      "\"" => vec!['"' as u8],
      "a" => vec!['\x07' as u8],
      "b" => vec!['\x08' as u8],
      "f" => vec!['\x0C' as u8],
      "n" => vec!['\n' as u8],
      "r" => vec!['\r' as u8],
      "t" => vec!['\t' as u8],
      "v" => vec!['\x0B' as u8],
      escaped =>
      {
         if OCT_ESCAPE_RE.is_match(escaped)
         {
            vec![u32::from_str_radix(escaped, 8).unwrap() as u8]
         }
         else if HEX_ESCAPE_RE.is_match(escaped)
         {
            vec![u32::from_str_radix(&escaped[1..], 16).unwrap() as u8]
         }
         else
         {
            let mut result = vec!['\\' as u8];
            push_all(&mut result, escaped.as_bytes());
            result
         }
      },
   }
}

// Temporary replacement for unstable
fn push_all(dest: &mut Vec<u8>, src: &[u8])
{
   for &v in src
   {
      dest.push(v)
   }
}

fn consume_space_to_next(text: &mut &str)
{
   match SPACE_RE.find(text)
   {
      None => (),
      Some((_, end)) => *text = &text[end..],
   }
}

fn determine_spaces(char_count: u32, tab_stop_size: u32)
   -> u32
{
   tab_stop_size - char_count % tab_stop_size
}

fn is_space(c: char)
   -> bool
{
   c == ' ' || c == '\t' || c == '\x0C'
}

fn process_character(count: u32, c: char)
   -> u32
{
   if c == '\t'
   {
      count + determine_spaces(count, TAB_STOP_SIZE)
   }
   else
   {
      count + 1
   }
}

fn count_indentation<'a>(line: &mut &'a str)
   -> u32
{
   let mut count = 0;
   let mut spaces = 0;

   for c in line.chars()
   {
      if is_space(c)
      {
         count = process_character(count, c);
         spaces += 1;
      }
      else
      {
         break;
      }
   }

   *line = &line[spaces..];
   count
}

/*
   -----------------------------------------------------------------
   --------------------- REGULAR EXPRESSIONS -----------------------
   -----------------------------------------------------------------
*/

lazy_static!
{
   static ref LOGICAL_EOL_RE : Regex = Regex::new(r"^$|^#.*(:?\r\n|\r|\n|$)|^\r\n|^\r|^\n").unwrap();
   static ref SPACE_RE : Regex = Regex::new(r"^[ \t\f]*").unwrap();
   static ref LINE_JOIN_START_RE : Regex = Regex::new(r"^\\").unwrap();
   static ref LINE_JOIN_RE : Regex = Regex::new(r"^\\(?:\r\n|\r|\n)").unwrap();
   static ref ID_RE : Regex =
      Regex::new(r"(?x)^
         [\p{Lu}\p{Ll}\p{Lt}\p{Lm}\p{Lo}\p{Nl}     # letters
            _
            \x{2118}\x{212E}\x{309B}\x{309C}       # Other_ID_Start
         ]
         [\p{Lu}\p{Ll}\p{Lt}\p{Lm}\p{Lo}\p{Nl}     # letters
            _
            \x{2118}\x{212E}\x{309B}\x{309C}       # Other_ID_Start
            \p{Mn}\p{Mc}\p{Nd}\p{Pc}               # Number and Connectors
            \x{00B7}\x{0387}\x{1369}-\x{1371}\x{19DA} # Other_ID_Continue
         ]*").unwrap();
   static ref BIN_RE : Regex = Regex::new(r"^0[bB][01]+").unwrap();
   static ref OCT_RE : Regex = Regex::new(r"^0[oO][0-7]+").unwrap();
   static ref HEX_RE : Regex = Regex::new(r"^0[xX][:xdigit:]+").unwrap();
   static ref DEC_RE : Regex = Regex::new(r"^0+|^[1-9]\d*").unwrap();
   static ref INVALID_DEC_RE : Regex = Regex::new(r"^0+[1-9]+").unwrap();
   static ref INVALID_ZERO_PRE_RE : Regex = Regex::new(r"^0[xX]|^0[bB]|^0[oO]").unwrap();
   static ref INT_IMG_RE : Regex = Regex::new(r"^\d+[jJ]").unwrap();
   static ref IMG_SUFFIX_RE : Regex = Regex::new(r"^[jJ]").unwrap();
   static ref FLOAT_RE : Regex =
      Regex::new(r"(?x)
         ^\d+[eE][\+-]?\d+     # dddddE+ddd
         | ^(?:
            \.\d+             # .ddddd
            | \d+\.(?:\d+)?   # dddddd. or ddddddd.ddddd
            )([eE][\+-]?\d+)?  # optionally E+ddddd
      ").unwrap();
   static ref SYMBOLS_RE : Regex = Regex::new(r"(?x)
      ^(?:\.\.\.|\.
         |\*\*=|\*\*|\*=|\*
         |<<=|<<|<=|<
         |>>=|>>|>=|>
         |//=|//|/=|/
         |\^=|\^
         |\|=|\|
         |&=|&
         |@=|@
         |%=|%
         |-=|->|-
         |\+=|\+
         |==|=
         |;|:|,|\{|\}|\[|\]|\(|\)|~|!=
       )
      ").unwrap();
   static ref STRING_START_RE : Regex =
      Regex::new(r#"^(?:[uU]|[rR])?['"]"#).unwrap();
   static ref STRING_PREFIX_RE : Regex =
      Regex::new(r#"^(?:[uU]|([rR]))?('''|'|"""|")"#).unwrap();
   static ref STRING_SINGLE_QUOTE_RE : Regex =
      Regex::new(r#"^(?s)((?:\\\r\n|\\.|[^\\\r\n'])*)'"#).unwrap();
   static ref STRING_DOUBLE_QUOTE_RE : Regex =
      Regex::new(r#"^(?s)((?:\\\r\n|\\.|[^\\\r\n"])*)""#).unwrap();
   static ref STRING_TRIPLE_SINGLE_QUOTE_RE : Regex =
      Regex::new(r#"^(?s)((?:[^\\]|\\.)*?)'''"#).unwrap();
   static ref STRING_TRIPLE_DOUBLE_QUOTE_RE : Regex =
      Regex::new(r#"^(?s)((?:[^\\]|\\.)*?)""""#).unwrap();
   static ref STRING_FAIL_RE : Regex =
      Regex::new(r#"^(?s)((?:[^\\\r\n]|\\.|\\\r\n)*)"#).unwrap();
   static ref STRING_TRIPLE_FAIL_RE : Regex =
      Regex::new(r#"^(?s)((?:[^\\]|\\.|\\\r\n)*?)$"#).unwrap();
   static ref NEWLINE_RE : Regex = Regex::new(r"\r\n|\r|\n").unwrap();
   static ref ESCAPES_RE : Regex =
      Regex::new(r#"\\(\r\n|\r|\n|\\|'|"|a|b|f|n|r|t|v|[0-7]{1,3}|x[:xdigit:]{2}|u[:xdigit:]{4}|U[:xdigit:]{8}|N\{[^\r\n\}]*\})"#).unwrap();
   static ref OCT_ESCAPE_RE : Regex = Regex::new("^[0-7]{1,3}").unwrap();
   static ref HEX_ESCAPE_RE : Regex = Regex::new("^x[:xdigit:]{2}").unwrap();
   static ref UNICODE_ESCAPE_RE : Regex =
      Regex::new("^(?:u[:xdigit:]{4}|U[:xdigit:]{8})").unwrap();
   static ref UNICODE_NAME_RE : Regex =
      Regex::new(r#"^N\{([^\r\n\}]*)\}"#).unwrap();
}
lazy_static!      // Recursion limit reached above o.O
{
   static ref UNICODE_NAMED_ESCAPE_RE : Regex =
      Regex::new(r#"\\N\{([^\r\n\}]*)\}"#).unwrap();
   static ref ESCAPES_FAIL_RE : Regex =
      Regex::new(r#"(?x)\\ (?:
         (?P<badu>u[:xdigit:]{0,3}(?:[:^xdigit:]|$))     # too few digits
         |(?P<badU>U[:xdigit:]{0,7}(?:[:^xdigit:]|$))    # too few digits
         |(?P<end>N\{[^\r\n\}]*(?:[\r\n]|$))             # missing end brace
         |(?P<start>N(?:[^\{]|$))                        # missing start brace
         |(?P<badx>x[:xdigit:]?(?:[:^xdigit:]|$))        # too few digits
      )"#).unwrap();
   static ref BYTES_START_RE : Regex =
      Regex::new(r#"^[bB][rR]?['"]|^[rR][bB]['"]"#).unwrap();
   static ref BYTES_PREFIX_RE : Regex =
      Regex::new(r#"^(?:[bB]([rR])?|([rR])[bB])('''|'|"""|")"#).unwrap();
   static ref NON_ASCII_RE : Regex =
      Regex::new(r"[:^ascii:]").unwrap();
   static ref ESCAPES_BYTES_RE : Regex =
      Regex::new(r#"\\(\r\n|\r|\n|\\|'|"|a|b|f|n|r|t|v|[0-7]{1,3}|x[:xdigit:]{2})"#).unwrap();
   static ref ESCAPES_BYTES_FAIL_RE : Regex =
      Regex::new(r#"(?x)\\
         (?P<badx>x[:xdigit:]?(?:[:^xdigit:]|$))        # too few digits
      "#).unwrap();
}

/*
   -----------------------------------------------------------------
   ----------------------- TESTS BELOW HERE ------------------------
   -----------------------------------------------------------------
*/

#[cfg(test)]
mod tests
{
   use super::Lexer;
   use super::tokens::Token;
   use super::errors::LexerError;

   #[test]
   fn test_identifiers()
   {
      let chars = "abf  \x0C _xyz\n   \n  e2f\n  \tmq3\nn12\\\r\nn3\\ \n  n23\n    n24\n   n25     # monkey says what?  \nafter_comment";
      let mut l = Lexer::new(chars);
      assert_eq!(l.next(), Some((1, Ok(Token::Identifier("abf".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::Identifier("_xyz".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::Newline))));
      assert_eq!(l.next(), Some((3, Ok(Token::Indent))));
      assert_eq!(l.next(), Some((3, Ok(Token::Identifier("e2f".to_owned())))));
      assert_eq!(l.next(), Some((3, Ok(Token::Newline))));
      assert_eq!(l.next(), Some((4, Ok(Token::Indent))));
      assert_eq!(l.next(), Some((4, Ok(Token::Identifier("mq3".to_owned())))));
      assert_eq!(l.next(), Some((4, Ok(Token::Newline))));
      assert_eq!(l.next(), Some((5, Ok(Token::Dedent))));
      assert_eq!(l.next(), Some((5, Ok(Token::Dedent))));
      assert_eq!(l.next(), Some((5, Ok(Token::Identifier("n12".to_owned())))));
      assert_eq!(l.next(), Some((6, Ok(Token::Identifier("n3".to_owned())))));
      assert_eq!(l.next(), Some((6, Err(LexerError::BadLineContinuation))));
      assert_eq!(l.next(), Some((6, Ok(Token::Newline))));
      assert_eq!(l.next(), Some((7, Ok(Token::Indent))));
      assert_eq!(l.next(), Some((7, Ok(Token::Identifier("n23".to_owned())))));
      assert_eq!(l.next(), Some((7, Ok(Token::Newline))));
      assert_eq!(l.next(), Some((8, Ok(Token::Indent))));
      assert_eq!(l.next(), Some((8, Ok(Token::Identifier("n24".to_owned())))));
      assert_eq!(l.next(), Some((8, Ok(Token::Newline))));
      assert_eq!(l.next(), Some((9, Err(LexerError::Dedent))));
      assert_eq!(l.next(), Some((9, Ok(Token::Identifier("n25".to_owned())))));
      assert_eq!(l.next(), Some((9, Ok(Token::Newline))));
      assert_eq!(l.next(), Some((10, Ok(Token::Dedent))));
      assert_eq!(l.next(), Some((10, Ok(Token::Identifier("after_comment".to_owned())))));
   }

   #[test]
   fn test_numbers()
   {
      let chars = "1 123 456 45 23.742 23. 12..3 .14 0123.2192 077e010 12e17 12e+17 12E-17 0 00000 00003 0.2 .e12 0o724 0X32facb7 0b10101010 0x 0b 0o9 00000e+00000 79228162514264337593543950336 0xdeadbeef 037j 2.3j 2.j .3j . 3..2\n";
      let mut l = Lexer::new(chars);
      assert_eq!(l.next(), Some((1, Ok(Token::DecInteger("1".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::DecInteger("123".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::DecInteger("456".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::DecInteger("45".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::Float("23.742".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::Float("23.".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::Float("12.".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::Float(".3".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::Float(".14".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::Float("0123.2192".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::Float("077e010".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::Float("12e17".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::Float("12e+17".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::Float("12E-17".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::DecInteger("0".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::DecInteger("00000".to_owned())))));
      assert_eq!(l.next(), Some((1, Err(LexerError::MalformedFloat))));
      assert_eq!(l.next(), Some((1, Ok(Token::Float("0.2".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::Dot))));
      assert_eq!(l.next(), Some((1, Ok(Token::Identifier("e12".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::OctInteger("0o724".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::HexInteger("0X32facb7".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::BinInteger("0b10101010".to_owned())))));
      assert_eq!(l.next(), Some((1, Err(LexerError::MissingDigits))));
      assert_eq!(l.next(), Some((1, Err(LexerError::MissingDigits))));
      assert_eq!(l.next(), Some((1, Err(LexerError::MissingDigits))));
      assert_eq!(l.next(), Some((1, Ok(Token::DecInteger("9".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::Float("00000e+00000".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::DecInteger("79228162514264337593543950336".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::HexInteger("0xdeadbeef".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::Imaginary("037j".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::Imaginary("2.3j".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::Imaginary("2.j".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::Imaginary(".3j".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::Dot))));
      assert_eq!(l.next(), Some((1, Ok(Token::Float("3.".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::Float(".2".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::Newline))));
   }

   #[test]
   fn test_dedent()
   {
      let chars = "    abf xyz\n\n\n\n        e2f\n             n12\n  n2\n";
      let mut l = Lexer::new(chars);
      assert_eq!(l.next(), Some((1, Ok(Token::Indent))));
      assert_eq!(l.next(), Some((1, Ok(Token::Identifier("abf".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::Identifier("xyz".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::Newline))));
      assert_eq!(l.next(), Some((5, Ok(Token::Indent))));
      assert_eq!(l.next(), Some((5, Ok(Token::Identifier("e2f".to_owned())))));
      assert_eq!(l.next(), Some((5, Ok(Token::Newline))));
      assert_eq!(l.next(), Some((6, Ok(Token::Indent))));
      assert_eq!(l.next(), Some((6, Ok(Token::Identifier("n12".to_owned())))));
      assert_eq!(l.next(), Some((6, Ok(Token::Newline))));
      assert_eq!(l.next(), Some((7, Ok(Token::Dedent))));
      assert_eq!(l.next(), Some((7, Ok(Token::Dedent))));
      assert_eq!(l.next(), Some((7, Err(LexerError::Dedent))));
      assert_eq!(l.next(), Some((7, Ok(Token::Identifier("n2".to_owned())))));
      assert_eq!(l.next(), Some((7, Ok(Token::Newline))));
   }

   #[test]
   fn test_symbols()
   {
      let chars = "(){}[]:,.;..===@->+=-=*=/=//=%=@=&=|=^=>>=<<=**=+-***///%@<<>>&|^~<><=>===!=!...$?`.";
      let mut l = Lexer::new(chars);
      assert_eq!(l.next(), Some((1, Ok(Token::Lparen))));
      assert_eq!(l.next(), Some((1, Ok(Token::Rparen))));
      assert_eq!(l.next(), Some((1, Ok(Token::Lbrace))));
      assert_eq!(l.next(), Some((1, Ok(Token::Rbrace))));
      assert_eq!(l.next(), Some((1, Ok(Token::Lbracket))));
      assert_eq!(l.next(), Some((1, Ok(Token::Rbracket))));
      assert_eq!(l.next(), Some((1, Ok(Token::Colon))));
      assert_eq!(l.next(), Some((1, Ok(Token::Comma))));
      assert_eq!(l.next(), Some((1, Ok(Token::Dot))));
      assert_eq!(l.next(), Some((1, Ok(Token::Semi))));
      assert_eq!(l.next(), Some((1, Ok(Token::Dot))));
      assert_eq!(l.next(), Some((1, Ok(Token::Dot))));
      assert_eq!(l.next(), Some((1, Ok(Token::EQ))));
      assert_eq!(l.next(), Some((1, Ok(Token::Assign))));
      assert_eq!(l.next(), Some((1, Ok(Token::At))));
      assert_eq!(l.next(), Some((1, Ok(Token::Arrow))));
      assert_eq!(l.next(), Some((1, Ok(Token::AssignPlus))));
      assert_eq!(l.next(), Some((1, Ok(Token::AssignMinus))));
      assert_eq!(l.next(), Some((1, Ok(Token::AssignTimes))));
      assert_eq!(l.next(), Some((1, Ok(Token::AssignDivide))));
      assert_eq!(l.next(), Some((1, Ok(Token::AssignDivideFloor))));
      assert_eq!(l.next(), Some((1, Ok(Token::AssignMod))));
      assert_eq!(l.next(), Some((1, Ok(Token::AssignAt))));
      assert_eq!(l.next(), Some((1, Ok(Token::AssignBitAnd))));
      assert_eq!(l.next(), Some((1, Ok(Token::AssignBitOr))));
      assert_eq!(l.next(), Some((1, Ok(Token::AssignBitXor))));
      assert_eq!(l.next(), Some((1, Ok(Token::AssignRshift))));
      assert_eq!(l.next(), Some((1, Ok(Token::AssignLshift))));
      assert_eq!(l.next(), Some((1, Ok(Token::AssignExponent))));
      assert_eq!(l.next(), Some((1, Ok(Token::Plus))));
      assert_eq!(l.next(), Some((1, Ok(Token::Minus))));
      assert_eq!(l.next(), Some((1, Ok(Token::Exponent))));
      assert_eq!(l.next(), Some((1, Ok(Token::Times))));
      assert_eq!(l.next(), Some((1, Ok(Token::DivideFloor))));
      assert_eq!(l.next(), Some((1, Ok(Token::Divide))));
      assert_eq!(l.next(), Some((1, Ok(Token::Mod))));
      assert_eq!(l.next(), Some((1, Ok(Token::At))));
      assert_eq!(l.next(), Some((1, Ok(Token::Lshift))));
      assert_eq!(l.next(), Some((1, Ok(Token::Rshift))));
      assert_eq!(l.next(), Some((1, Ok(Token::BitAnd))));
      assert_eq!(l.next(), Some((1, Ok(Token::BitOr))));
      assert_eq!(l.next(), Some((1, Ok(Token::BitXor))));
      assert_eq!(l.next(), Some((1, Ok(Token::BitNot))));
      assert_eq!(l.next(), Some((1, Ok(Token::LT))));
      assert_eq!(l.next(), Some((1, Ok(Token::GT))));
      assert_eq!(l.next(), Some((1, Ok(Token::LE))));
      assert_eq!(l.next(), Some((1, Ok(Token::GE))));
      assert_eq!(l.next(), Some((1, Ok(Token::EQ))));
      assert_eq!(l.next(), Some((1, Ok(Token::NE))));
      assert_eq!(l.next(), Some((1, Err(LexerError::InvalidSymbol("!".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::Ellipsis))));
      assert_eq!(l.next(), Some((1, Err(LexerError::InvalidSymbol("$".to_owned())))));
      assert_eq!(l.next(), Some((1, Err(LexerError::InvalidSymbol("?".to_owned())))));
      assert_eq!(l.next(), Some((1, Err(LexerError::InvalidSymbol("`".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::Dot))));
   }

   #[test]
   fn test_keywords()
   {
      let chars = "false False None True and as assert async await break class continue def del defdel elif else except finally for from \nglobal if import in is lambda nonlocal not or pass raise return try while with yield\n";
      let mut l = Lexer::new(chars);
      assert_eq!(l.next(), Some((1, Ok(Token::Identifier("false".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::False))));
      assert_eq!(l.next(), Some((1, Ok(Token::None))));
      assert_eq!(l.next(), Some((1, Ok(Token::True))));
      assert_eq!(l.next(), Some((1, Ok(Token::And))));
      assert_eq!(l.next(), Some((1, Ok(Token::As))));
      assert_eq!(l.next(), Some((1, Ok(Token::Assert))));
      //assert_eq!(l.next(), Some((1, Ok(Token::Async))));
      //assert_eq!(l.next(), Some((1, Ok(Token::Await))));
      assert_eq!(l.next(), Some((1, Ok(Token::Identifier("async".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::Identifier("await".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::Break))));
      assert_eq!(l.next(), Some((1, Ok(Token::Class))));
      assert_eq!(l.next(), Some((1, Ok(Token::Continue))));
      assert_eq!(l.next(), Some((1, Ok(Token::Def))));
      assert_eq!(l.next(), Some((1, Ok(Token::Del))));
      assert_eq!(l.next(), Some((1, Ok(Token::Identifier("defdel".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::Elif))));
      assert_eq!(l.next(), Some((1, Ok(Token::Else))));
      assert_eq!(l.next(), Some((1, Ok(Token::Except))));
      assert_eq!(l.next(), Some((1, Ok(Token::Finally))));
      assert_eq!(l.next(), Some((1, Ok(Token::For))));
      assert_eq!(l.next(), Some((1, Ok(Token::From))));
      assert_eq!(l.next(), Some((1, Ok(Token::Newline))));
      assert_eq!(l.next(), Some((2, Ok(Token::Global))));
      assert_eq!(l.next(), Some((2, Ok(Token::If))));
      assert_eq!(l.next(), Some((2, Ok(Token::Import))));
      assert_eq!(l.next(), Some((2, Ok(Token::In))));
      assert_eq!(l.next(), Some((2, Ok(Token::Is))));
      assert_eq!(l.next(), Some((2, Ok(Token::Lambda))));
      assert_eq!(l.next(), Some((2, Ok(Token::Nonlocal))));
      assert_eq!(l.next(), Some((2, Ok(Token::Not))));
      assert_eq!(l.next(), Some((2, Ok(Token::Or))));
      assert_eq!(l.next(), Some((2, Ok(Token::Pass))));
      assert_eq!(l.next(), Some((2, Ok(Token::Raise))));
      assert_eq!(l.next(), Some((2, Ok(Token::Return))));
      assert_eq!(l.next(), Some((2, Ok(Token::Try))));
      assert_eq!(l.next(), Some((2, Ok(Token::While))));
      assert_eq!(l.next(), Some((2, Ok(Token::With))));
      assert_eq!(l.next(), Some((2, Ok(Token::Yield))));
      assert_eq!(l.next(), Some((2, Ok(Token::Newline))));
   }

   #[test]
   fn test_strings_1()
   {
      let chars = "'abc 123 \txyz@\")#*)@'\n\"wfe wf w fwe'fwefw\"\n\"abc\n'last line'\n'just\\\n   kidding   \\\n \t kids'\n'xy\\\n  zq\nxyz'";
      let mut l = Lexer::new(chars);
      assert_eq!(l.next(), Some((1, Ok(Token::String("abc 123 \txyz@\")#*)@".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::Newline))));
      assert_eq!(l.next(), Some((2, Ok(Token::String("wfe wf w fwe'fwefw".to_owned())))));
      assert_eq!(l.next(), Some((2, Ok(Token::Newline))));
      assert_eq!(l.next(), Some((3, Err(LexerError::UnterminatedString))));
      assert_eq!(l.next(), Some((3, Ok(Token::Newline))));
      assert_eq!(l.next(), Some((4, Ok(Token::String("last line".to_owned())))));
      assert_eq!(l.next(), Some((4, Ok(Token::Newline))));
      assert_eq!(l.next(), Some((5, Ok(Token::String("just   kidding    \t kids".to_owned())))));
      assert_eq!(l.next(), Some((7, Ok(Token::Newline))));
      assert_eq!(l.next(), Some((9, Err(LexerError::UnterminatedString))));
   }

   #[test]
   fn test_strings_2()
   {
      let chars = "'abc' \"def\" \\\n'123'\n";
      let mut l = Lexer::new(chars);
      assert_eq!(l.next(), Some((1, Ok(Token::String("abcdef123".to_owned())))));
      assert_eq!(l.next(), Some((2, Ok(Token::Newline))));
   }

   #[test]
   fn test_strings_3()
   {
      let chars = "''' abc ' '' '''\n\"\"\"xyz\"\"\"\n'''abc\n \tdef\n123'''\n'''abc\\\n \tdef\\\n123'''\n'''abc\ndef";
      let mut l = Lexer::new(chars);
      assert_eq!(l.next(), Some((1, Ok(Token::String(" abc ' '' ".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::Newline))));
      assert_eq!(l.next(), Some((2, Ok(Token::String("xyz".to_owned())))));
      assert_eq!(l.next(), Some((2, Ok(Token::Newline))));
      assert_eq!(l.next(), Some((3, Ok(Token::String("abc\n \tdef\n123".to_owned())))));
      assert_eq!(l.next(), Some((5, Ok(Token::Newline))));
      assert_eq!(l.next(), Some((6, Ok(Token::String("abc \tdef123".to_owned())))));
      assert_eq!(l.next(), Some((8, Ok(Token::Newline))));
      assert_eq!(l.next(), Some((10, Err(LexerError::UnterminatedTripleString))));
   }

   #[test]
   fn test_strings_4()
   {
      let chars = "'\\\\'\n'\\''\n'\\\"'\n'\\a'\n'\\b'\n'\\f'\n'\\n'\n'\\r'\n'\\t'\n'\\v'\n'\\m'";
      let mut l = Lexer::new(chars);
      assert_eq!(l.next(), Some((1, Ok(Token::String("\\".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::Newline))));
      assert_eq!(l.next(), Some((2, Ok(Token::String("'".to_owned())))));
      assert_eq!(l.next(), Some((2, Ok(Token::Newline))));
      assert_eq!(l.next(), Some((3, Ok(Token::String("\"".to_owned())))));
      assert_eq!(l.next(), Some((3, Ok(Token::Newline))));
      assert_eq!(l.next(), Some((4, Ok(Token::String("\x07".to_owned())))));
      assert_eq!(l.next(), Some((4, Ok(Token::Newline))));
      assert_eq!(l.next(), Some((5, Ok(Token::String("\x08".to_owned())))));
      assert_eq!(l.next(), Some((5, Ok(Token::Newline))));
      assert_eq!(l.next(), Some((6, Ok(Token::String("\x0C".to_owned())))));
      assert_eq!(l.next(), Some((6, Ok(Token::Newline))));
      assert_eq!(l.next(), Some((7, Ok(Token::String("\n".to_owned())))));
      assert_eq!(l.next(), Some((7, Ok(Token::Newline))));
      assert_eq!(l.next(), Some((8, Ok(Token::String("\r".to_owned())))));
      assert_eq!(l.next(), Some((8, Ok(Token::Newline))));
      assert_eq!(l.next(), Some((9, Ok(Token::String("\t".to_owned())))));
      assert_eq!(l.next(), Some((9, Ok(Token::Newline))));
      assert_eq!(l.next(), Some((10, Ok(Token::String("\x0B".to_owned())))));
      assert_eq!(l.next(), Some((10, Ok(Token::Newline))));
      assert_eq!(l.next(), Some((11, Ok(Token::String("\\m".to_owned())))));
   }

   #[test]
   fn test_strings_5()
   {
      let chars = "'\\007'\n'\\7'\n'\\175'\n'\\x07'\n'\\1750'\n";
      let mut l = Lexer::new(chars);
      assert_eq!(l.next(), Some((1, Ok(Token::String("\x07".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::Newline))));
      assert_eq!(l.next(), Some((2, Ok(Token::String("\x07".to_owned())))));
      assert_eq!(l.next(), Some((2, Ok(Token::Newline))));
      assert_eq!(l.next(), Some((3, Ok(Token::String("}".to_owned())))));
      assert_eq!(l.next(), Some((3, Ok(Token::Newline))));
      assert_eq!(l.next(), Some((4, Ok(Token::String("\x07".to_owned())))));
      assert_eq!(l.next(), Some((4, Ok(Token::Newline))));
      assert_eq!(l.next(), Some((5, Ok(Token::String("}0".to_owned())))));
      assert_eq!(l.next(), Some((5, Ok(Token::Newline))));
   }

   #[test]
   fn test_strings_6()
   {
      let chars = "'\\x'";
      let mut l = Lexer::new(chars);
      assert_eq!(l.next(), Some((1, Err(LexerError::HexEscapeShort))));
   }

   #[test]
   fn test_strings_7()
   {
      let chars = "'\\x7'";
      let mut l = Lexer::new(chars);
      assert_eq!(l.next(), Some((1, Err(LexerError::HexEscapeShort))));
   }

   #[test]
   fn test_strings_8()
   {
      let chars = "'\\N{monkey}'\n'\\N{BLACK STAR}'";
      let mut l = Lexer::new(chars);
      assert_eq!(l.next(), Some((1, Ok(Token::String("🐒".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::Newline))));
      assert_eq!(l.next(), Some((2, Ok(Token::String("★".to_owned())))));
   }

   #[test]
   fn test_strings_9()
   {
      let chars = "'\\N{monkey'";
      let mut l = Lexer::new(chars);
      assert_eq!(l.next(), Some((1, Err(LexerError::MalformedNamedUnicodeEscape))));
   }

   #[test]
   fn test_strings_10()
   {
      let chars = "'\\Nmonkey'";
      let mut l = Lexer::new(chars);
      assert_eq!(l.next(), Some((1, Err(LexerError::MalformedNamedUnicodeEscape))));
   }

   #[test]
   fn test_strings_11()
   {
      let chars = "'\\N{fhefaefi}'";
      let mut l = Lexer::new(chars);
      assert_eq!(l.next(), Some((1, Err(LexerError::UnknownUnicodeName("fhefaefi".to_owned())))));
   }

   #[test]
   fn test_strings_12()
   {
      let chars = "'\\u262f'\n'\\U00002D5E'";
      let mut l = Lexer::new(chars);
      assert_eq!(l.next(), Some((1, Ok(Token::String("☯".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::Newline))));
      assert_eq!(l.next(), Some((2, Ok(Token::String("ⵞ".to_owned())))));
   }

   #[test]
   fn test_strings_13()
   {
      let chars = "'\\u262'";
      let mut l = Lexer::new(chars);
      assert_eq!(l.next(), Some((1, Err(LexerError::MalformedUnicodeEscape))));
   }

   #[test]
   fn test_strings_14()
   {
      let chars = "'\\U00002D'";
      let mut l = Lexer::new(chars);
      assert_eq!(l.next(), Some((1, Err(LexerError::MalformedUnicodeEscape))));
   }

   #[test]
   fn test_strings_15()
   {
      let chars = "'\\u262f262f'";
      let mut l = Lexer::new(chars);
      assert_eq!(l.next(), Some((1, Ok(Token::String("☯262f".to_owned())))));
   }

   #[test]
   fn test_strings_16()
   {
      let chars = "unlikely u'abc' u '123' U\"\"\"def\"\"\" u\n";
      let mut l = Lexer::new(chars);
      assert_eq!(l.next(), Some((1, Ok(Token::Identifier("unlikely".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::String("abc".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::Identifier("u".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::String("123def".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::Identifier("u".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::Newline))));
   }

   #[test]
   fn test_strings_17()
   {
      let chars = "r'\\txyz \\\n \\'fefe \\N{monkey}'";
      let mut l = Lexer::new(chars);
      assert_eq!(l.next(), Some((1, Ok(Token::String("\\txyz \\\n \\'fefe \\N{monkey}".to_owned())))));
   }

   #[test]
   fn test_strings_18()
   {
      let chars = "r'''\\txyz \\\n \\'fefe \\N{monkey}''''hello\\040\\700\\300'";
      let mut l = Lexer::new(chars);
      assert_eq!(l.next(), Some((1, Ok(Token::String("\\txyz \\\n \\'fefe \\N{monkey}hello ǀÀ".to_owned())))));
   }

   #[test]
   fn test_strings_19()
   {
      let chars = "'''hello\\\n";
      let mut l = Lexer::new(chars);
      assert_eq!(l.next(), Some((2, Err(LexerError::UnterminatedTripleString))));
   }

   #[test]
   fn test_strings_20()
   {
      let chars = "'\\N'";
      let mut l = Lexer::new(chars);
      assert_eq!(l.next(), Some((1, Err(LexerError::MalformedNamedUnicodeEscape))));
   }

   #[test]
   fn test_strings_21()
   {
      let chars = "'\\x7y'";
      let mut l = Lexer::new(chars);
      assert_eq!(l.next(), Some((1, Err(LexerError::HexEscapeShort))));
   }

   #[test]
   fn test_strings_22()
   {
      let chars = "'\\N{monkey}\\N{fhefaefi}'";
      let mut l = Lexer::new(chars);
      assert_eq!(l.next(), Some((1, Err(LexerError::UnknownUnicodeName("fhefaefi".to_owned())))));
   }

   #[test]
   fn test_strings_23()
   {
      let chars = "'\\N{monkey}\\x\\\n'a";
      let mut l = Lexer::new(chars);
      assert_eq!(l.next(), Some((1, Err(LexerError::HexEscapeShort))));
      assert_eq!(l.next(), Some((2, Ok(Token::Identifier("a".to_owned())))));
   }

   #[test]
   fn test_strings_24()
   {
      let chars = "'\\N'";
      let mut l = Lexer::new(chars);
      assert_eq!(l.next(), Some((1, Err(LexerError::MalformedNamedUnicodeEscape))));
   }

   #[test]
   fn test_byte_strings_1()
   {
      let chars = "b'''hello'''";
      let mut l = Lexer::new(chars);
      assert_eq!(l.next(), Some((1, Ok(Token::Bytes(vec![104, 101, 108, 108, 111])))));
   }

   #[test]
   fn test_byte_strings_2()
   {
      let chars = "b'''hello\nblah'''";
      let mut l = Lexer::new(chars);
      assert_eq!(l.next(), Some((1, Ok(Token::Bytes(vec![104, 101, 108, 108, 111, 10, 98, 108, 97, 104])))));
   }

   #[test]
   fn test_byte_strings_3()
   {
      let chars = "b'\\x26\\040'";
      let mut l = Lexer::new(chars);
      assert_eq!(l.next(), Some((1, Ok(Token::Bytes(vec![38, 32])))));
   }

   #[test]
   fn test_byte_strings_4()
   {
      let chars = "b'\\x26\\040\\700\\300'";
      let mut l = Lexer::new(chars);
      assert_eq!(l.next(), Some((1, Ok(Token::Bytes(vec![38, 32, 192, 192])))));
   }

   #[test]
   fn test_byte_strings_5()
   {
      let chars = "b'abc\\\n  \t 123'";
      let mut l = Lexer::new(chars);
      assert_eq!(l.next(), Some((1, Ok(Token::Bytes(vec![97, 98, 99, 32, 32, 9, 32, 49, 50, 51])))));
   }

   #[test]
   fn test_byte_strings_6()
   {
      let chars = "b'abc\\\n  \t 123' \\\n  b'123'";
      let mut l = Lexer::new(chars);
      assert_eq!(l.next(), Some((1, Ok(Token::Bytes(vec![97, 98, 99, 32, 32, 9, 32, 49, 50, 51, 49, 50, 51])))));
   }

   #[test]
   fn test_byte_strings_7()
   {
      let chars = "rb'abc\\' \\\n  \t 123'";
      let mut l = Lexer::new(chars);
      assert_eq!(l.next(), Some((1, Ok(Token::Bytes(vec![97, 98, 99, 92, 39, 32, 92, 10, 32, 32, 9, 32, 49, 50, 51])))));
   }

   #[test]
   fn test_byte_strings_8()
   {
      let chars = "Br'abc\\' \\\n  \t' bR' 123'";
      let mut l = Lexer::new(chars);
      assert_eq!(l.next(), Some((1, Ok(Token::Bytes(vec![97, 98, 99, 92, 39, 32, 92, 10, 32, 32, 9, 32, 49, 50, 51])))));
   }

   #[test]
   fn test_byte_strings_9()
   {
      let chars = "b'\\x'";
      let mut l = Lexer::new(chars);
      assert_eq!(l.next(), Some((1, Err(LexerError::HexEscapeShort))));
   }

   #[test]
   fn test_byte_strings_10()
   {
      let chars = "b'\\x7'";
      let mut l = Lexer::new(chars);
      assert_eq!(l.next(), Some((1, Err(LexerError::HexEscapeShort))));
   }

   #[test]
   fn test_byte_strings_11()
   {
      let chars = "b'\\N{monkey}'";
      let mut l = Lexer::new(chars);
      assert_eq!(l.next(), Some((1, Ok(Token::Bytes(vec![92, 78, 123, 109, 111, 110, 107, 101, 121, 125])))));
   }

   #[test]
   fn test_byte_strings_12()
   {
      let chars = "b'abdafe ef a efw fw🐒feafe f \\\nwf we fw'\n";
      let mut l = Lexer::new(chars);
      assert_eq!(l.next(), Some((1, Err(LexerError::BytesNonASCII))));
      assert_eq!(l.next(), Some((2, Ok(Token::Newline))));
   }

   #[test]
   fn test_implicit_1()
   {
      let chars = "(1 + \n      2 \n)";
      let mut l = Lexer::new(chars);
      assert_eq!(l.next(), Some((1, Ok(Token::Lparen))));
      assert_eq!(l.next(), Some((1, Ok(Token::DecInteger("1".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::Plus))));
      assert_eq!(l.next(), Some((2, Ok(Token::DecInteger("2".to_owned())))));
      assert_eq!(l.next(), Some((3, Ok(Token::Rparen))));
   }

   #[test]
   fn test_implicit_2()
   {
      let chars = "   (1 + \n   (   2 \n + 9 \n ) * \n      2 \n )\n2";
      let mut l = Lexer::new(chars);
      assert_eq!(l.next(), Some((1, Ok(Token::Indent))));
      assert_eq!(l.next(), Some((1, Ok(Token::Lparen))));
      assert_eq!(l.next(), Some((1, Ok(Token::DecInteger("1".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::Plus))));
      assert_eq!(l.next(), Some((2, Ok(Token::Lparen))));
      assert_eq!(l.next(), Some((2, Ok(Token::DecInteger("2".to_owned())))));
      assert_eq!(l.next(), Some((3, Ok(Token::Plus))));
      assert_eq!(l.next(), Some((3, Ok(Token::DecInteger("9".to_owned())))));
      assert_eq!(l.next(), Some((4, Ok(Token::Rparen))));
      assert_eq!(l.next(), Some((4, Ok(Token::Times))));
      assert_eq!(l.next(), Some((5, Ok(Token::DecInteger("2".to_owned())))));
      assert_eq!(l.next(), Some((6, Ok(Token::Rparen))));
      assert_eq!(l.next(), Some((6, Ok(Token::Newline))));
      assert_eq!(l.next(), Some((7, Ok(Token::Dedent))));
      assert_eq!(l.next(), Some((7, Ok(Token::DecInteger("2".to_owned())))));
   }

   #[test]
   fn test_implicit_3()
   {
      let chars = "('abc' \n      'def' \n)";
      let mut l = Lexer::new(chars);
      assert_eq!(l.next(), Some((1, Ok(Token::Lparen))));
      assert_eq!(l.next(), Some((1, Ok(Token::String("abcdef".to_owned())))));
      assert_eq!(l.next(), Some((3, Ok(Token::Rparen))));
   }

   #[test]
   fn test_implicit_4()
   {
      let chars = "def abc(a, g,\n         c):\n   first";
      let mut l = Lexer::new(chars);
      assert_eq!(l.next(), Some((1, Ok(Token::Def))));
      assert_eq!(l.next(), Some((1, Ok(Token::Identifier("abc".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::Lparen))));
      assert_eq!(l.next(), Some((1, Ok(Token::Identifier("a".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::Comma))));
      assert_eq!(l.next(), Some((1, Ok(Token::Identifier("g".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::Comma))));
      assert_eq!(l.next(), Some((2, Ok(Token::Identifier("c".to_owned())))));
      assert_eq!(l.next(), Some((2, Ok(Token::Rparen))));
      assert_eq!(l.next(), Some((2, Ok(Token::Colon))));
      assert_eq!(l.next(), Some((2, Ok(Token::Newline))));
      assert_eq!(l.next(), Some((3, Ok(Token::Indent))));
      assert_eq!(l.next(), Some((3, Ok(Token::Identifier("first".to_owned())))));
      assert_eq!(l.next(), Some((0, Ok(Token::Dedent))));
   }

   #[test]
   fn test_implicit_5()
   {
      let chars = "('abc'\n   #  'def' \n)";
      let mut l = Lexer::new(chars);
      assert_eq!(l.next(), Some((1, Ok(Token::Lparen))));
      assert_eq!(l.next(), Some((1, Ok(Token::String("abc".to_owned())))));
      assert_eq!(l.next(), Some((3, Ok(Token::Rparen))));
   }

   #[test]
   fn test_implicit_6()
   {
      let chars = "'abc'\n   #  'def' \n123\n";
      let mut l = Lexer::new(chars);
      assert_eq!(l.next(), Some((1, Ok(Token::String("abc".to_owned())))));
      assert_eq!(l.next(), Some((1, Ok(Token::Newline))));
      assert_eq!(l.next(), Some((3, Ok(Token::DecInteger("123".to_owned())))));
      assert_eq!(l.next(), Some((3, Ok(Token::Newline))));
      assert_eq!(l.next(), None);
   }
}
