pub type LineNumber = u16;
pub type ColumnNumber = usize;
pub type CharacterPosition = usize;

#[derive(Debug, PartialEq, Copy, Clone)]
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

  // Blank
  Comment,
  Whitespace,
  EndOfLine,
  EndOfFile,
  Unknown,
}

#[derive(Debug)]
pub struct Token<'source> {
  pub ttype: TokenType,
  pub value: &'source str,

  pub start: CharacterPosition,
  pub end: CharacterPosition,

  pub line: LineNumber,
  pub column: ColumnNumber,
}

struct Tokeniser<'source> {
  source: &'source [u8],

  line: LineNumber,
  column: ColumnNumber,
  position: CharacterPosition,
}

impl<'source> Tokeniser<'source> {
  pub fn new(source: &'source str) -> Tokeniser<'source> {
    Tokeniser {
      source: source.as_bytes(),

      line: 1,
      column: 0,
      position: 0,
    }
  }

  pub fn tokens_left(&self) -> bool {
    self.position < self.source.len()
  }

  pub fn next_token(&mut self) -> Token<'source> {
    let (token_type, length) = self.next_token_type();
    let value = unsafe {
      // This is safe because `self.source` is converted from a string and not mutated.
      std::str::from_utf8_unchecked(&self.source[self.position..self.position + length])
    };

    let token = Token {
      ttype: token_type,
      start: self.position,
      end: self.position + length,
      line: self.line,
      column: self.column,
      value,
    };

    self.position += length;
    if token.ttype == TokenType::EndOfLine {
      self.line += 1;
      self.column = 0;
    } else {
      self.column += length;
    }

    token
  }

  fn next_token_type(&mut self) -> (TokenType, CharacterPosition) {
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
      b'\n' => (TokenType::EndOfLine, 1),
      b'(' => (TokenType::LeftParen, 1),
      b')' => (TokenType::RightParen, 1),
      b'{' => (TokenType::LeftBrace, 1),
      b'}' => (TokenType::RightBrace, 1),
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
      _ => (TokenType::Unknown, 1),
    }
  }

  fn at_end(&self, position: CharacterPosition) -> bool {
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
      _ => None,
    }
  }

  fn whitespace(&self) -> (TokenType, CharacterPosition) {
    let mut position = self.position;

    while !self.at_end(position) && matches!(self.source[position], b' ' | b'\t' | b'\r') {
      position += 1;
    }

    (TokenType::Whitespace, position - self.position)
  }

  fn comment(&self) -> (TokenType, CharacterPosition) {
    let mut position = self.position + 2;

    while !self.at_end(position) && self.source[position] != b'\n' {
      position += 1;
    }

    (TokenType::Comment, position - self.position)
  }

  fn string(&self, quote: u8) -> (TokenType, CharacterPosition) {
    let mut position = self.position + 1;

    while !self.at_end(position) && self.source[position] != quote {
      position += 1;
    }

    if self.at_end(position) {
      (TokenType::String, position - self.position)
    } else {
      (TokenType::String, (position - self.position) + 1)
    }
  }

  fn number(&self) -> (TokenType, CharacterPosition) {
    let mut position = self.position + 1;

    while !self.at_end(position) && matches!(self.source[position], b'0'..=b'9' | b'_') {
      position += 1;
    }

    if !self.at_end(position)
      && self.source[position] == b'.'
      && matches!(self.source[position + 1], b'0'..=b'9')
    {
      position += 1;
    }

    while !self.at_end(position) && matches!(self.source[position], b'0'..=b'9' | b'_') {
      position += 1;
    }

    (TokenType::Number, position - self.position)
  }

  fn identifier(&self) -> (TokenType, CharacterPosition) {
    let mut position = self.position;

    while !self.at_end(position + 1)
      && matches!(self.source[position + 1], b'_' | b'a'..=b'z' | b'A'..=b'Z')
    {
      position += 1;
    }

    let length = (position - self.position) + 1;
    (self.identifier_type(length), length)
  }

  fn identifier_type(&self, length: CharacterPosition) -> TokenType {
    match self.source[self.position] {
      b'a' => self.check_keyword(length, "and", TokenType::And),
      b'e' => self.check_keyword(length, "else", TokenType::Else),
      b'f' => match self.source[self.position + 1] {
        b'a' => self.check_keyword(length, "false", TokenType::False),
        b'u' => self.check_keyword(length, "fun", TokenType::Fun),
        _ => TokenType::Identifier,
      },
      b'i' => self.check_keyword(length, "if", TokenType::If),
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
    length: CharacterPosition,
    keyword: &'static str,
    token_type: TokenType,
  ) -> TokenType {
    if &self.source[self.position..self.position + length] == keyword.as_bytes() {
      token_type
    } else {
      TokenType::Identifier
    }
  }
}

pub fn tokenize(source: &str) -> Vec<Token> {
  let mut tokeniser = Tokeniser::new(source);
  let mut tokens = Vec::with_capacity(source.len() / 5);

  while tokeniser.tokens_left() {
    tokens.push(tokeniser.next_token());
  }

  tokens
}

#[cfg(test)]
mod tests {
  use super::*;

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
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].ttype, TokenType::Whitespace);

    let tokens = tokenize("\t");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].ttype, TokenType::Whitespace);

    let tokens = tokenize("\r");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].ttype, TokenType::Whitespace);

    let tokens = tokenize(" \r \t ");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].ttype, TokenType::Whitespace);

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
        column: 0,
        value: "'hello'"
      }
    ));

    let tokens = tokenize("`world`");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].ttype, TokenType::String);
    assert_eq!(tokens[0].value, "`world`");

    let tokens = tokenize("\"What's Up\"");
    assert_eq!(tokens[0].value, "\"What's Up\"");

    let tokens = tokenize("\"\n        What's\n        Up\"");
    assert_eq!(tokens[0].value, "\"\n        What's\n        Up\"");

    let tokens = tokenize("'hello' `world`");
    assert_eq!(tokens.len(), 3);
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
        column: 0,
        value: "752"
      }
    ));

    let tokens = tokenize("1.5");
    assert_eq!(tokens[0].ttype, TokenType::Number);
    assert_eq!(tokens[0].value, "1.5");

    let tokens = tokenize(".75");
    assert_eq!(tokens[0].ttype, TokenType::Number);
    assert_eq!(tokens[0].value, ".75");

    let tokens = tokenize("32_175.45");
    assert_eq!(tokens[0].ttype, TokenType::Number);
    assert_eq!(tokens[0].value, "32_175.45");

    let tokens = tokenize("32_175.4__5");
    assert_eq!(tokens[0].ttype, TokenType::Number);
    assert_eq!(tokens[0].value, "32_175.4__5");
  }

  #[test]
  fn should_tokenize_keywords() {
    let tokens = tokenize("and else false fun if let null or return true while");
    assert_eq!(tokens.len(), 21);
    assert_eq!(tokens[0].ttype, TokenType::And);
    assert_eq!(tokens[2].ttype, TokenType::Else);
    assert_eq!(tokens[4].ttype, TokenType::False);
    assert_eq!(tokens[6].ttype, TokenType::Fun);
    assert_eq!(tokens[8].ttype, TokenType::If);
    assert_eq!(tokens[10].ttype, TokenType::Let);
    assert_eq!(tokens[12].ttype, TokenType::Null);
    assert_eq!(tokens[14].ttype, TokenType::Or);
    assert_eq!(tokens[16].ttype, TokenType::Return);
    assert_eq!(tokens[18].ttype, TokenType::True);
    assert_eq!(tokens[20].ttype, TokenType::While);
  }

  #[test]
  fn should_tokenize_identifiers() {
    let tokens = tokenize("hello");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].ttype, TokenType::Identifier);
    assert_eq!(tokens[0].value, "hello");

    let tokens = tokenize("_hello");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].ttype, TokenType::Identifier);
    assert_eq!(tokens[0].value, "_hello");

    let tokens = tokenize("hello_");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].ttype, TokenType::Identifier);
    assert_eq!(tokens[0].value, "hello_");

    let tokens = tokenize("_hello_");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].ttype, TokenType::Identifier);
    assert_eq!(tokens[0].value, "_hello_");

    let tokens = tokenize("hello_world");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].ttype, TokenType::Identifier);
    assert_eq!(tokens[0].value, "hello_world");
  }

  #[test]
  fn should_tokenize_unknown_characters() {
    let tokens = tokenize("?");
    assert_eq!(tokens[0].ttype, TokenType::Unknown);

    let tokens = tokenize("#");
    assert_eq!(tokens[0].ttype, TokenType::Unknown);

    let tokens = tokenize("&");
    assert_eq!(tokens[0].ttype, TokenType::Unknown);
  }
}
