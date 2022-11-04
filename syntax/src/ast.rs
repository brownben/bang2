use crate::tokens::{CharacterPosition, LineNumber, Token};

pub mod expression;
pub mod statement;
pub mod types;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Span {
  pub start: CharacterPosition,
  pub end: CharacterPosition,
}
impl Span {
  pub fn get_line_number(&self, source: &str) -> LineNumber {
    let mut line: LineNumber = 1;

    for (i, byte) in source.as_bytes().iter().enumerate() {
      if *byte == b'\n' {
        line += 1;
      }

      if i == self.start as usize {
        return line;
      }
    }

    line
  }

  pub fn get_line_number_end(&self, source: &str) -> LineNumber {
    let mut line: LineNumber = 1;

    for (i, byte) in source.as_bytes().iter().enumerate() {
      if i == self.end as usize {
        return line;
      }

      if *byte == b'\n' {
        line += 1;
      }
    }

    line
  }
}
impl From<Token> for Span {
  fn from(token: Token) -> Self {
    Self {
      start: token.start,
      end: token.end,
    }
  }
}
