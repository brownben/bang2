use crate::error::Error;

pub type LineNumber = u16;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenType {
  // Brackets
  LeftParen,
  RightParen,
  LeftBrace,
  RightBrace,

  // Separators
  Comma,
  Dot,
  Colon,
  RightArrow,
  FatRightArrow,

  // Operators
  Minus,
  Plus,
  Slash,
  Star,
  Bang,
  And,
  Or,
  QuestionQuestion,

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

  // Keywords
  Else,
  Fun,
  If,
  Let,
  Return,
  While,

  BlockStart,
  BlockEnd,

  Blank,
  Error,
  EndOfLine,
  EndOfFile,
}

#[derive(Debug, Clone, Copy)]
pub struct Token {
  pub token_type: TokenType,
  pub line: LineNumber,

  pub start: usize,
  pub end: usize,

  pub error_value: Option<Error>,
}

impl Token {
  pub fn get_value(&self, chars: &[char]) -> String {
    chars[self.start..self.end].iter().collect()
  }

  pub fn get_value_from_string(&self, string: &str) -> String {
    String::from(string).chars().collect::<Vec<char>>()[self.start..self.end]
      .iter()
      .collect()
  }
}
