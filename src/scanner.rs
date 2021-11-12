pub type LineNumber = u16;
pub type MaxSourceLength = usize;

use crate::error::Error;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum TokenType {
  // Brackets
  LeftParen,
  RightParen,
  LeftBrace,
  RightBrace,

  // Separators
  Comma,
  Dot,

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
  Print,
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
  pub start: MaxSourceLength,
  pub end: MaxSourceLength,
  pub error_value: Option<Error>,
}

impl Token {
  pub fn get_value(&self, scanner: &Scanner) -> String {
    scanner
      .chars
      .get(self.start..self.end)
      .unwrap()
      .iter()
      .collect::<String>()
  }
}

pub struct Scanner {
  pub chars: Vec<char>,
  pub from: String,

  start: MaxSourceLength,
  current: MaxSourceLength,
  line: LineNumber,

  last_token_type: TokenType,
  indentation: u8,
}

impl Scanner {
  fn at_end(&self) -> bool {
    self.current as usize >= self.chars.len()
  }

  fn advance(&mut self) -> char {
    self.current += 1;
    self.chars[(self.current as usize) - 1]
  }

  fn peek(&self) -> Option<&char> {
    self.chars.get(self.current as usize)
  }

  fn peek_equals(&self, expected: char) -> bool {
    match self.peek() {
      Some(c) => *c == expected,
      _ => false,
    }
  }

  fn peek_next(&self) -> Option<&char> {
    self.chars.get((self.current as usize) + 1)
  }

  fn peek_start_next_line(&self) -> Option<&char> {
    let mut i: usize = self.current;
    while let Some('\r' | '\t' | ' ' | '\n') = self.chars.get(i) {
      i += 1;
    }

    self.chars.get(i)
  }

  fn is_next_line_comment(&self) -> bool {
    let mut i: usize = self.current;
    while let Some('\r' | '\t' | ' ' | '\n') = self.chars.get(i) {
      i += 1;
    }

    matches!(self.chars.get(i), Some('/')) && matches!(self.chars.get(i + 1), Some('/'))
  }

  pub fn new(source: &str, from: String) -> Self {
    Self {
      chars: source.chars().collect(),
      from: from,
      start: 0,
      current: 0,
      line: 1,
      last_token_type: TokenType::Blank,
      indentation: 0,
    }
  }

  pub fn get_token(&mut self) -> Token {
    let token = self.find_token();
    self.last_token_type = token.token_type;
    token
  }

  fn find_token(&mut self) -> Token {
    if self.last_token_type == TokenType::EndOfLine && !self.is_next_line_comment() {
      let indentation = get_indentation(self);
      if indentation > self.indentation {
        self.indentation = indentation;
        return make_token(self, TokenType::BlockStart);
      } else if indentation < self.indentation {
        self.indentation = indentation;
        self.start = self.current;
        return make_token(self, TokenType::BlockEnd);
      }
    } else {
      skip_whitespace(self, false);
    }

    let is_end_of_line = self.peek_equals('\n')
      && is_valid_line_end_token(self.last_token_type)
      && !is_invalid_line_start_character(self);

    if !is_end_of_line && self.peek_equals('\n') {
      skip_whitespace(self, true)
    }

    self.start = self.current;

    if is_end_of_line {
      newline_token(self)
    } else if self.at_end() {
      make_token(self, TokenType::EndOfFile)
    } else {
      let character = self.advance();
      let next_character = self.peek();

      if let Some(token_type) = get_two_character_token(character, next_character) {
        self.advance();
        make_token(self, token_type)
      } else {
        match character {
          '0'..='9' => number_token(self),
          '.' if matches!(next_character, Some('0'..='9')) => number_token(self),
          '_' | 'a'..='z' | 'A'..='Z' => identifier_token(self),
          quote @ ('"' | '\'' | '`') => string_token(self, quote),
          '(' => make_token(self, TokenType::LeftParen),
          ')' => make_token(self, TokenType::RightParen),
          '{' => make_token(self, TokenType::LeftBrace),
          '}' => make_token(self, TokenType::RightBrace),
          ',' => make_token(self, TokenType::Comma),
          '.' => make_token(self, TokenType::Dot),
          '-' => make_token(self, TokenType::Minus),
          '+' => make_token(self, TokenType::Plus),
          '/' => make_token(self, TokenType::Slash),
          '*' => make_token(self, TokenType::Star),
          '!' => make_token(self, TokenType::Bang),
          '=' => make_token(self, TokenType::Equal),
          '<' => make_token(self, TokenType::Less),
          '>' => make_token(self, TokenType::Greater),
          _ => error_token(self, Error::UnknownCharacter),
        }
      }
    }
  }
}

fn get_two_character_token(char1: char, char2: Option<&char>) -> Option<TokenType> {
  match (char1, char2?) {
    ('!', '=') => Some(TokenType::BangEqual),
    ('=', '=') => Some(TokenType::EqualEqual),
    ('<', '=') => Some(TokenType::LessEqual),
    ('>', '=') => Some(TokenType::GreaterEqual),
    ('&', '&') => Some(TokenType::And),
    ('|', '|') => Some(TokenType::Or),
    ('+', '=') => Some(TokenType::PlusEqual),
    ('-', '=') => Some(TokenType::MinusEqual),
    ('*', '=') => Some(TokenType::StarEqual),
    ('/', '=') => Some(TokenType::SlashEqual),
    ('?', '?') => Some(TokenType::QuestionQuestion),
    _ => None,
  }
}

fn make_token(scanner: &Scanner, token_type: TokenType) -> Token {
  Token {
    token_type: token_type,
    line: scanner.line,
    error_value: None,
    start: scanner.start,
    end: scanner.current,
  }
}

fn error_token(scanner: &Scanner, error: Error) -> Token {
  Token {
    token_type: TokenType::Error,
    line: scanner.line,
    error_value: Some(error),
    start: scanner.start,
    end: scanner.current,
  }
}

fn skip_whitespace(scanner: &mut Scanner, skip_newlines: bool) {
  loop {
    match scanner.peek() {
      // Skip Newlines if they are irrelevant
      Some('\n') if skip_newlines => {
        scanner.advance();
        scanner.line += 1;
      }
      //  Ignore whitespace
      Some(' ' | '\t' | '\r') => {
        scanner.advance();
      }
      // Skip Comments
      Some('/') => match scanner.peek_next() {
        Some('/') => {
          while !scanner.peek_equals('\n') && !scanner.at_end() {
            scanner.advance();
          }
          if scanner.peek_equals('\n') {
            scanner.advance();
            scanner.line += 1;
          }
        }
        _ => break,
      },
      _ => break,
    };
  }
}

fn get_indentation(scanner: &mut Scanner) -> u8 {
  let mut spaces = 0;
  loop {
    match scanner.peek() {
      Some(' ') => {
        spaces += 1;
        scanner.advance()
      }
      Some('\t') => {
        spaces += 2;
        scanner.advance()
      }
      Some('\r') => scanner.advance(),
      _ => break,
    };
  }
  spaces >> 1
}

fn newline_token(scanner: &mut Scanner) -> Token {
  scanner.advance();
  scanner.start = scanner.current;
  let token = make_token(scanner, TokenType::EndOfLine);

  // Link newline with the content before it
  scanner.line += 1;

  token
}

fn string_token(scanner: &mut Scanner, quote: char) -> Token {
  while !scanner.peek_equals(quote) && !scanner.at_end() {
    if scanner.peek_equals('\n') && quote == '`' {
      scanner.line += 1
    } else if scanner.peek_equals('\n') {
      return error_token(scanner, Error::UnterminatedString);
    }

    scanner.advance();
  }

  if scanner.at_end() {
    error_token(scanner, Error::UnterminatedString)
  } else {
    scanner.advance(); // closing quote
    make_token(scanner, TokenType::String)
  }
}

fn number_token(scanner: &mut Scanner) -> Token {
  while is_digit(scanner.peek()) || scanner.peek_equals('_') {
    scanner.advance();
  }

  if scanner.peek_equals('.') && is_digit(scanner.peek_next()) {
    scanner.advance();
  }

  while is_digit(scanner.peek()) || scanner.peek_equals('_') {
    scanner.advance();
  }

  make_token(scanner, TokenType::Number)
}

fn identifier_token(scanner: &mut Scanner) -> Token {
  while is_alpha(scanner.peek()) || is_digit(scanner.peek()) {
    scanner.advance();
  }

  make_token(scanner, identifier_type(scanner))
}

fn identifier_type(scanner: &Scanner) -> TokenType {
  match scanner.chars[scanner.start] {
    'a' => check_keyword(scanner, "and", TokenType::And),
    'e' => check_keyword(scanner, "else", TokenType::Else),
    'f' => match scanner.chars[scanner.start + 1] {
      'a' => check_keyword(scanner, "false", TokenType::False),
      'u' => check_keyword(scanner, "fun", TokenType::Fun),
      _ => TokenType::Identifier,
    },
    'i' => check_keyword(scanner, "if", TokenType::If),
    'l' => check_keyword(scanner, "let", TokenType::Let),
    'n' => check_keyword(scanner, "null", TokenType::Null),
    'o' => check_keyword(scanner, "or", TokenType::Or),
    'p' => check_keyword(scanner, "print", TokenType::Print),
    'r' => check_keyword(scanner, "return", TokenType::Return),
    't' => check_keyword(scanner, "true", TokenType::True),
    'w' => check_keyword(scanner, "while", TokenType::While),
    _ => TokenType::Identifier,
  }
}

fn check_keyword(scanner: &Scanner, rest: &str, token_type: TokenType) -> TokenType {
  let string: String = scanner.chars[scanner.start..scanner.start + rest.len()]
    .iter()
    .collect();

  if string == *rest {
    token_type
  } else {
    TokenType::Identifier
  }
}

fn is_digit(c: Option<&char>) -> bool {
  match c {
    Some('0'..='9') => true,
    _ => false,
  }
}

fn is_alpha(c: Option<&char>) -> bool {
  match c {
    Some('_' | 'a'..='z' | 'A'..='Z') => true,
    _ => false,
  }
}

fn is_invalid_line_start_character(scanner: &Scanner) -> bool {
  if scanner.is_next_line_comment() {
    false
  } else {
    match scanner.peek_start_next_line() {
      Some(')' | ']' | '.' | ',' | '*' | '/' | '+' | '<' | '>' | '=' | '&' | '|' | ':' | '}') => {
        true
      }
      _ => false,
    }
  }
}

fn is_valid_line_end_token(token_type: TokenType) -> bool {
  match token_type {
    TokenType::RightParen
    | TokenType::RightBrace
    | TokenType::Identifier
    | TokenType::String
    | TokenType::Number
    | TokenType::True
    | TokenType::False
    | TokenType::Null
    | TokenType::Return => true,
    _ => false,
  }
}

// Print Tokens
#[cfg(feature = "debug-token")]
pub fn print_tokens(source: &str, from: String) {
  let mut scanner = Scanner::new(source, from);
  let mut line = 0;

  println!("     ╭─[Tokens]");
  loop {
    let token = scanner.get_token();
    if token.line != line {
      print!("{:>4} │ ", token.line);
      line = token.line;
    } else {
      print!("     │ ");
    }
    print!("{:?} ({})\n", token.token_type, token.get_value(&scanner));

    if token.token_type == TokenType::EndOfFile {
      print!("─────╯\n");
      break;
    }
  }
}
