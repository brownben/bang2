use smallvec::SmallVec;
use std::str;

pub type LineNumber = u16;
pub type CharacterPosition = u32;
type TokenLength = usize;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TokenType {
  // Brackets
  LeftParen,
  RightParen,
  LeftBrace,
  RightBrace,
  LeftSquare,
  RightSquare,

  // Separators
  Comma,
  Dot,
  Colon,
  ColonColon,
  RightArrow,
  FatRightArrow,
  DotDot,

  // Type Operators
  Pipe,
  Question,

  // Operators
  Minus,
  Plus,
  Slash,
  Star,
  Bang,
  And,
  Or,
  QuestionQuestion,
  RightRight,

  // Comparators
  BangEqual,
  Equal,
  EqualEqual,
  Greater,
  GreaterEqual,
  Less,
  LessEqual,

  // Assignment Operators
  PlusEqual,
  MinusEqual,
  StarEqual,
  SlashEqual,

  // Values
  Identifier,
  String,
  Number,
  True,
  False,
  Null,

  // Format String
  FormatStringStart,
  FormatStringPart,
  FormatStringEnd,

  // Keywords
  As,
  Else,
  If,
  Import,
  From,
  Let,
  Return,
  While,

  // Blank
  Comment,
  Whitespace,
  EndOfLine,
  EndOfFile,
  Unknown,
}
impl TokenType {
  pub fn is_assignment_operator(self) -> bool {
    matches!(
      self,
      Self::PlusEqual | Self::MinusEqual | Self::StarEqual | Self::SlashEqual
    )
  }
}

#[derive(Clone, Copy, Debug)]
pub struct Token {
  pub ttype: TokenType,
  pub start: CharacterPosition,
  pub end: CharacterPosition,
  pub line: LineNumber,
}
impl Token {
  pub fn get_value<'s>(&self, source: &'s [u8]) -> &'s str {
    let start = self.start as usize;
    let end = self.end as usize;

    str::from_utf8(&source[start..end]).expect("Source to be valid utf8")
  }

  pub fn len(&self) -> CharacterPosition {
    self.end - self.start
  }
}
impl Default for Token {
  fn default() -> Self {
    Self {
      ttype: TokenType::EndOfFile,
      start: 0,
      end: 0,
      line: 0,
    }
  }
}

pub struct Tokeniser<'source> {
  source: &'source [u8],

  line: LineNumber,
  position: usize,

  quote_stack: SmallVec<[u8; 8]>,
  last_type: TokenType,
}

impl<'source> Tokeniser<'source> {
  pub fn new(source: &'source str) -> Tokeniser<'source> {
    Tokeniser {
      source: source.as_bytes(),

      line: 1,
      position: 0,

      quote_stack: SmallVec::new(),
      last_type: TokenType::Unknown,
    }
  }

  fn next_token(&mut self) -> Token {
    let (ttype, len) = self.next_token_type();

    #[allow(clippy::cast_possible_truncation)]
    // assume files are less than 2^32 characters
    let token = Token {
      ttype,
      start: self.position as CharacterPosition,
      end: (self.position + len) as CharacterPosition,
      line: self.line,
    };

    self.position += len;
    if token.ttype == TokenType::EndOfLine {
      self.line += 1;
    }

    token
  }

  fn next_token_type(&mut self) -> (TokenType, TokenLength) {
    let character = &self.source[self.position];
    let next_character = self.source.get(self.position + 1);

    if let Some(token_type) = self.two_character_token() {
      return (token_type, 2);
    }

    match character {
      quote @ (b'"' | b'\'' | b'`') => self.string(*quote),
      b'0'..=b'9' => self.number(),
      b' ' | b'\r' | b'\t' => self.whitespace(),
      b'_' | b'a'..=b'z' | b'A'..=b'Z' => self.identifier(),
      b'/' if matches!(next_character, Some(b'/')) => self.comment(),
      b'.' if matches!(next_character, Some(b'0'..=b'9')) => self.number(),
      b'}' if !self.quote_stack.is_empty() => self.format_string(),
      b'\n' => (TokenType::EndOfLine, 1),
      b'(' => (TokenType::LeftParen, 1),
      b')' => (TokenType::RightParen, 1),
      b'{' => (TokenType::LeftBrace, 1),
      b'}' => (TokenType::RightBrace, 1),
      b'[' => (TokenType::LeftSquare, 1),
      b']' => (TokenType::RightSquare, 1),
      b',' => (TokenType::Comma, 1),
      b'.' => (TokenType::Dot, 1),
      b'+' => (TokenType::Plus, 1),
      b'-' => (TokenType::Minus, 1),
      b'/' => (TokenType::Slash, 1),
      b'*' => (TokenType::Star, 1),
      b'!' => (TokenType::Bang, 1),
      b'=' => (TokenType::Equal, 1),
      b'<' => (TokenType::Less, 1),
      b'>' => (TokenType::Greater, 1),
      b':' => (TokenType::Colon, 1),
      b'|' => (TokenType::Pipe, 1),
      b'?' => (TokenType::Question, 1),
      _ => (TokenType::Unknown, 1),
    }
  }

  fn at_end(&self, position: usize) -> bool {
    position >= self.source.len()
  }

  fn two_character_token(&self) -> Option<TokenType> {
    let character = &self.source[self.position];
    let next_character = self.source.get(self.position + 1);

    match (character, next_character?) {
      (b'!', b'=') => Some(TokenType::BangEqual),
      (b'=', b'=') => Some(TokenType::EqualEqual),
      (b'<', b'=') => Some(TokenType::LessEqual),
      (b'>', b'=') => Some(TokenType::GreaterEqual),
      (b'+', b'=') => Some(TokenType::PlusEqual),
      (b'-', b'=') => Some(TokenType::MinusEqual),
      (b'*', b'=') => Some(TokenType::StarEqual),
      (b'/', b'=') => Some(TokenType::SlashEqual),
      (b'-', b'>') => Some(TokenType::RightArrow),
      (b'=', b'>') => Some(TokenType::FatRightArrow),
      (b'&', b'&') => Some(TokenType::And),
      (b'|', b'|') => Some(TokenType::Or),
      (b'?', b'?') => Some(TokenType::QuestionQuestion),
      (b'>', b'>') => Some(TokenType::RightRight),
      (b'.', b'.') => Some(TokenType::DotDot),
      (b':', b':') => Some(TokenType::ColonColon),
      _ => None,
    }
  }

  fn whitespace(&self) -> (TokenType, TokenLength) {
    let mut position = self.position;

    while !self.at_end(position) && matches!(self.source[position], b' ' | b'\t' | b'\r') {
      position += 1;
    }

    (TokenType::Whitespace, position - self.position)
  }

  fn comment(&self) -> (TokenType, TokenLength) {
    let mut position = self.position + 2;

    while !self.at_end(position) && self.source[position] != b'\n' {
      position += 1;
    }

    (TokenType::Comment, position - self.position)
  }

  fn string(&mut self, quote: u8) -> (TokenType, TokenLength) {
    let mut pos = self.position + 1;

    loop {
      if self.at_end(pos) {
        break (TokenType::String, pos - self.position);
      } else if self.source[pos] == quote {
        break (TokenType::String, pos - self.position + 1);
      } else if self.source[pos] == b'$' && self.source.get(pos + 1) == Some(&b'{') {
        self.quote_stack.push(quote);
        break (TokenType::FormatStringStart, pos - self.position + 2);
      }

      if self.source[pos] == b'\n' {
        self.line += 1;
      }

      pos += 1;
    }
  }

  fn format_string(&mut self) -> (TokenType, TokenLength) {
    let quote = *self.quote_stack.last().unwrap();
    let mut pos = self.position + 1;

    loop {
      if self.at_end(pos) {
        break (TokenType::FormatStringEnd, pos - self.position);
      } else if self.source[pos] == quote {
        self.quote_stack.pop();
        break (TokenType::FormatStringEnd, pos - self.position + 1);
      } else if self.source[pos] == b'$' && self.source.get(pos + 1) == Some(&b'{') {
        break (TokenType::FormatStringPart, pos - self.position + 2);
      }

      if self.source[pos] == b'\n' {
        self.line += 1;
      }

      pos += 1;
    }
  }

  fn number(&self) -> (TokenType, TokenLength) {
    let mut position = self.position + 1;

    while !self.at_end(position) && matches!(self.source[position], b'0'..=b'9' | b'_') {
      position += 1;
    }

    if !self.at_end(position)
      && self.source[position] == b'.'
      && self.source[position + 1].is_ascii_digit()
    {
      position += 1;
    }

    while !self.at_end(position) && matches!(self.source[position], b'0'..=b'9' | b'_') {
      position += 1;
    }

    (TokenType::Number, position - self.position)
  }

  fn identifier(&self) -> (TokenType, TokenLength) {
    let mut position = self.position;

    while !self.at_end(position + 1)
      && (self.source[position + 1].is_ascii_alphanumeric() || self.source[position + 1] == b'_')
    {
      position += 1;
    }

    let length = position - self.position + 1;
    (self.identifier_type(length), length)
  }

  fn identifier_type(&self, length: TokenLength) -> TokenType {
    match self.source[self.position] {
      b'a' => match self.source.get(self.position + 1) {
        Some(b'n') => self.check_keyword(length, "and", TokenType::And),
        Some(b's') => self.check_keyword(length, "as", TokenType::As),
        _ => TokenType::Identifier,
      },
      b'e' => self.check_keyword(length, "else", TokenType::Else),
      b'f' => match self.source.get(self.position + 1) {
        Some(b'a') => self.check_keyword(length, "false", TokenType::False),
        Some(b'r') => self.check_keyword(length, "from", TokenType::From),
        _ => TokenType::Identifier,
      },
      b'i' => match self.source.get(self.position + 1) {
        Some(b'f') => self.check_keyword(length, "if", TokenType::If),
        Some(b'm') => self.check_keyword(length, "import", TokenType::Import),
        _ => TokenType::Identifier,
      },
      b'l' => self.check_keyword(length, "let", TokenType::Let),
      b'n' => self.check_keyword(length, "null", TokenType::Null),
      b'o' => self.check_keyword(length, "or", TokenType::Or),
      b'r' => self.check_keyword(length, "return", TokenType::Return),
      b't' => self.check_keyword(length, "true", TokenType::True),
      b'w' => self.check_keyword(length, "while", TokenType::While),
      _ => TokenType::Identifier,
    }
  }

  fn check_keyword(
    &self,
    length: TokenLength,
    keyword: &'static str,
    token_type: TokenType,
  ) -> TokenType {
    let end = self.position + length;
    if &self.source[self.position..end] == keyword.as_bytes() {
      token_type
    } else {
      TokenType::Identifier
    }
  }
}

impl Iterator for Tokeniser<'_> {
  type Item = Token;

  fn next(&mut self) -> Option<Self::Item> {
    if self.at_end(self.position) {
      return None;
    }

    let token = self.next_token();
    if token.ttype == TokenType::Whitespace && self.last_type != TokenType::EndOfLine {
      self.next()
    } else {
      self.last_type = token.ttype;
      Some(token)
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn tokenize(source: &str) -> Vec<Token> {
    Tokeniser::new(source).collect()
  }

  #[test]
  fn should_have_no_tokens_for_empty_string() {
    let tokens = tokenize("");

    assert_eq!(tokens.len(), 0);
  }

  #[test]
  fn should_tokenize_single_character() {
    let tokens = tokenize("(");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].ttype, TokenType::LeftParen);

    let tokens = tokenize(".");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].ttype, TokenType::Dot);

    let tokens = tokenize("+");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].ttype, TokenType::Plus);
  }

  #[test]
  fn should_tokenize_whitespace() {
    let tokens = tokenize(" ");
    assert_eq!(tokens.len(), 0);

    let tokens = tokenize("\t");
    assert_eq!(tokens.len(), 0);

    let tokens = tokenize("\r");
    assert_eq!(tokens.len(), 0);

    let tokens = tokenize(" \r \t ");
    assert_eq!(tokens.len(), 0);

    let tokens = tokenize("\n ");
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].ttype, TokenType::EndOfLine);
    assert_eq!(tokens[0].line, 1);
    assert_eq!(tokens[1].ttype, TokenType::Whitespace);
    assert_eq!(tokens[1].line, 2);
  }

  #[test]
  fn should_tokenize_strings() {
    let tokens = tokenize("'hello'");
    assert_eq!(tokens.len(), 1);
    assert!(matches!(
      tokens[0],
      Token {
        ttype: TokenType::String,
        start: 0,
        end: 7,
        line: 1,
      }
    ));

    let tokens = tokenize("`world`");
    assert_eq!(tokens[0].ttype, TokenType::String);
    assert_eq!(tokens.len(), 1);

    let tokens = tokenize("\"What's Up\"");
    assert_eq!(tokens[0].ttype, TokenType::String);
    assert_eq!(tokens.len(), 1);

    let tokens = tokenize("\"\n        What's\n        Up\"");
    assert_eq!(tokens[0].ttype, TokenType::String);
    assert_eq!(tokens.len(), 1);

    let tokens = tokenize("'hello' `world`");
    assert_eq!(tokens.len(), 2);
  }

  #[test]
  fn should_tokenize_format_strings() {
    let tokens = tokenize("'${}'");
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].ttype, TokenType::FormatStringStart);
    assert_eq!(tokens[1].ttype, TokenType::FormatStringEnd);

    let tokens = tokenize("'a ${}'");
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].ttype, TokenType::FormatStringStart);
    assert_eq!(tokens[1].ttype, TokenType::FormatStringEnd);

    let tokens = tokenize("'a ${3}'");
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0].ttype, TokenType::FormatStringStart);
    assert_eq!(tokens[1].ttype, TokenType::Number);
    assert_eq!(tokens[2].ttype, TokenType::FormatStringEnd);

    let tokens = tokenize("'a ${1} b ${false} c'");
    assert_eq!(tokens.len(), 5);
    assert_eq!(tokens[0].ttype, TokenType::FormatStringStart);
    assert_eq!(tokens[1].ttype, TokenType::Number);
    assert_eq!(tokens[2].ttype, TokenType::FormatStringPart);
    assert_eq!(tokens[3].ttype, TokenType::False);
    assert_eq!(tokens[4].ttype, TokenType::FormatStringEnd);
    assert_eq!(tokens[0].len(), 5);
    assert_eq!(tokens[1].len(), 1);
    assert_eq!(tokens[2].len(), 6);
    assert_eq!(tokens[3].len(), 5);
    assert_eq!(tokens[4].len(), 4);
  }

  #[test]
  fn should_tokenize_numbers() {
    let tokens = tokenize("752");
    assert_eq!(tokens.len(), 1);
    assert!(matches!(
      tokens[0],
      Token {
        ttype: TokenType::Number,
        start: 0,
        end: 3,
        line: 1,
      }
    ));

    let tokens = tokenize("1.5");
    assert_eq!(tokens[0].ttype, TokenType::Number);
    assert_eq!(tokens.len(), 1);

    let tokens = tokenize(".75");
    assert_eq!(tokens[0].ttype, TokenType::Number);
    assert_eq!(tokens.len(), 1);

    let tokens = tokenize("32_175.45");
    assert_eq!(tokens[0].ttype, TokenType::Number);
    assert_eq!(tokens.len(), 1);

    let tokens = tokenize("32_175.4__5");
    assert_eq!(tokens[0].ttype, TokenType::Number);
    assert_eq!(tokens.len(), 1);
  }

  #[test]
  fn should_tokenize_keywords() {
    let tokens = tokenize("and else false if let null or return true while");
    assert_eq!(tokens.len(), 10);
    assert_eq!(tokens[0].ttype, TokenType::And);
    assert_eq!(tokens[1].ttype, TokenType::Else);
    assert_eq!(tokens[2].ttype, TokenType::False);
    assert_eq!(tokens[3].ttype, TokenType::If);
    assert_eq!(tokens[4].ttype, TokenType::Let);
    assert_eq!(tokens[5].ttype, TokenType::Null);
    assert_eq!(tokens[6].ttype, TokenType::Or);
    assert_eq!(tokens[7].ttype, TokenType::Return);
    assert_eq!(tokens[8].ttype, TokenType::True);
    assert_eq!(tokens[9].ttype, TokenType::While);
  }

  #[test]
  fn should_tokenize_identifiers() {
    let tokens = tokenize("hello");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].ttype, TokenType::Identifier);

    let tokens = tokenize("_hello");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].ttype, TokenType::Identifier);

    let tokens = tokenize("hello_");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].ttype, TokenType::Identifier);

    let tokens = tokenize("_hello_");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].ttype, TokenType::Identifier);

    let tokens = tokenize("hello_world");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].ttype, TokenType::Identifier);

    let tokens = tokenize("hello_3");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].ttype, TokenType::Identifier);

    let tokens = tokenize("3hello");
    assert_eq!(tokens.len(), 2);
  }

  #[test]
  fn should_tokenize_unknown_characters() {
    let tokens = tokenize("#");
    assert_eq!(tokens[0].ttype, TokenType::Unknown);

    let tokens = tokenize("&");
    assert_eq!(tokens[0].ttype, TokenType::Unknown);
  }
}
