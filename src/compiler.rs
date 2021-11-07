use crate::chunk::{Chunk, OpCode};
use crate::scanner::{Scanner, Token, TokenType};
use crate::value::Value;

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

#[cfg(feature = "debug")]
use crate::chunk;
#[cfg(feature = "debug")]
use crate::scanner;

#[derive(Debug, FromPrimitive, PartialOrd, PartialEq)]
enum Precedence {
  None = 1,
  Assignment, // =
  Or,         // or
  And,        // and
  Nullish,    // ??
  Equality,   // == !=
  Comparison, // < > <= >=
  Term,       // + -
  Factor,     // * /
  Unary,      // ! -
  Call,       // . ()
  Primary,
}

fn get_precedence(number: u8) -> Precedence {
  match FromPrimitive::from_u8(number) {
    Some(precedence) => precedence,
    _ => Precedence::None,
  }
}

type ParseFn = fn(parser: &mut Parser, can_assign: bool);

struct ParseRule {
  pub prefix: Option<ParseFn>,
  pub infix: Option<ParseFn>,
  pub precedence: Precedence,
}

#[derive(Debug)]
struct Local {
  name: String,
  depth: u8,
}

struct Parser {
  scanner: Scanner,
  chunk: Chunk,

  current: Option<Token>,
  previous: Option<Token>,

  had_error: bool,
  panic_mode: bool,

  locals: Vec<Local>,
  scope_depth: u8,
}

impl Parser {
  fn new(source: &str) -> Self {
    Self {
      scanner: Scanner::new(source),
      chunk: Chunk::new(),

      current: None,
      previous: None,

      had_error: false,
      panic_mode: false,

      locals: Vec::new(),
      scope_depth: 0,
    }
  }

  fn advance(&mut self) {
    self.previous = self.current.take();

    loop {
      let token = self.scanner.get_token();
      self.current = Some(token);

      match self.current.unwrap().token_type {
        TokenType::Error => self.error_at_current(&self.current.unwrap().get_error()),
        _ => break,
      };
    }
  }

  fn begin_scope(&mut self) {
    self.scope_depth += 1;
  }

  fn end_scope(&mut self) {
    self.scope_depth -= 1;
  }

  fn matches(&mut self, token_type: TokenType) -> bool {
    if self.current.unwrap().token_type == token_type {
      self.advance();
      true
    } else {
      false
    }
  }

  fn error_at_current(&mut self, message: &str) {
    self.error_at(self.current, message);
  }

  fn error(&mut self, message: &str) {
    self.error_at(self.previous, message);
  }

  fn error_at(&mut self, token: Option<Token>, message: &str) {
    if !self.panic_mode {
      match token {
        Some(token) => println!("[Line={}] Error: {}", token.line, message),
        _ => println!("Error: {}", message),
      }
    }

    self.panic_mode = true;
    self.had_error = true;
  }

  fn consume(&mut self, token_type: TokenType, message: &str) {
    if self.current.is_none() || self.current.unwrap().token_type != token_type {
      self.error_at_current(message);
    } else {
      self.advance();
    }
  }

  fn end_compiler(&mut self) {
    #[cfg(feature = "debug")]
    {
      chunk::disassemble(&self.chunk, "Bytecode");
      println!();
    }

    self.emit_opcode(OpCode::Return);
    self.chunk.finalize();
  }
}

// Emit Bytecode
impl Parser {
  fn emit_opcode(&mut self, code: OpCode) {
    match &self.previous {
      Some(token) => self.chunk.write(code, token.line),
      _ => self.chunk.write(code, 0),
    }
  }

  fn emit_value(&mut self, value: u8) {
    match &self.previous {
      Some(token) => self.chunk.write_value(value, token.line),
      _ => self.chunk.write_value(value, 0),
    }
  }

  fn emit_long_value(&mut self, value: u16) {
    match &self.previous {
      Some(token) => self.chunk.write_long_value(value, token.line),
      _ => self.chunk.write_long_value(value, 0),
    }
  }

  fn emit_constant(&mut self, value: Value) {
    let constant_position = self.chunk.add_constant(value);

    if constant_position <= u8::max_value() as usize {
      self.emit_opcode(OpCode::Constant);
      self.emit_value(constant_position as u8);
    } else if constant_position <= u16::max_value() as usize {
      self.emit_opcode(OpCode::ConstantLong);
      self.emit_long_value(constant_position as u16);
    } else {
      self.error("Too many constants in one chunk.");
    }
  }

  fn emit_constant_string(&mut self, value: String) {
    let constant_position = self.chunk.add_constant_string(value);

    if constant_position <= u8::max_value() as usize {
      self.emit_value(constant_position as u8);
    } else {
      self.error("Too many constants in one chunk.");
    }
  }
}

pub fn compile(source: &str) -> (Chunk, bool) {
  #[cfg(feature = "debug")]
  {
    scanner::print_tokens(source);
  }

  let mut parser = Parser::new(source);

  parser.advance();

  while !parser.matches(TokenType::EndOfFile) {
    declaration(&mut parser);
  }

  parser.consume(TokenType::EndOfFile, "Expect end of expression.");
  parser.end_compiler();

  (parser.chunk, !parser.had_error)
}

fn get_rule(token_type: TokenType) -> ParseRule {
  match token_type {
    TokenType::LeftParen => ParseRule {
      prefix: Some(grouping),
      infix: None,
      precedence: Precedence::None,
    },

    TokenType::Plus => ParseRule {
      prefix: None,
      infix: Some(binary),
      precedence: Precedence::Term,
    },
    TokenType::Minus => ParseRule {
      prefix: Some(unary),
      infix: Some(binary),
      precedence: Precedence::Term,
    },
    TokenType::Star => ParseRule {
      prefix: None,
      infix: Some(binary),
      precedence: Precedence::Factor,
    },
    TokenType::Slash => ParseRule {
      prefix: None,
      infix: Some(binary),
      precedence: Precedence::Factor,
    },
    TokenType::Bang => ParseRule {
      prefix: Some(unary),
      infix: None,
      precedence: Precedence::None,
    },

    TokenType::BangEqual => ParseRule {
      prefix: None,
      infix: Some(binary),
      precedence: Precedence::Equality,
    },
    TokenType::EqualEqual => ParseRule {
      prefix: None,
      infix: Some(binary),
      precedence: Precedence::Equality,
    },
    TokenType::Greater => ParseRule {
      prefix: None,
      infix: Some(binary),
      precedence: Precedence::Comparison,
    },
    TokenType::GreaterEqual => ParseRule {
      prefix: None,
      infix: Some(binary),
      precedence: Precedence::Comparison,
    },
    TokenType::Less => ParseRule {
      prefix: None,
      infix: Some(binary),
      precedence: Precedence::Comparison,
    },
    TokenType::LessEqual => ParseRule {
      prefix: None,
      infix: Some(binary),
      precedence: Precedence::Comparison,
    },

    TokenType::Identifier => ParseRule {
      prefix: Some(variable),
      infix: None,
      precedence: Precedence::None,
    },
    TokenType::String => ParseRule {
      prefix: Some(string),
      infix: None,
      precedence: Precedence::None,
    },
    TokenType::Number => ParseRule {
      prefix: Some(number),
      infix: None,
      precedence: Precedence::None,
    },

    TokenType::And => ParseRule {
      prefix: None,
      infix: Some(and),
      precedence: Precedence::And,
    },
    TokenType::Or => ParseRule {
      prefix: None,
      infix: Some(or),
      precedence: Precedence::Or,
    },
    TokenType::QuestionQuestion => ParseRule {
      prefix: None,
      infix: Some(nullish),
      precedence: Precedence::Nullish,
    },

    TokenType::Null => ParseRule {
      prefix: Some(literal),
      infix: None,
      precedence: Precedence::None,
    },
    TokenType::True => ParseRule {
      prefix: Some(literal),
      infix: None,
      precedence: Precedence::None,
    },
    TokenType::False => ParseRule {
      prefix: Some(literal),
      infix: None,
      precedence: Precedence::None,
    },

    _ => ParseRule {
      prefix: None,
      infix: None,
      precedence: Precedence::None,
    },
  }
}

fn synchronize(parser: &mut Parser) {
  parser.panic_mode = false;

  while parser.current.unwrap().token_type != TokenType::EndOfFile {
    if parser.previous.unwrap().token_type == TokenType::EndOfLine {
      return;
    }

    match parser.current.unwrap().token_type {
      TokenType::Fun => return,
      TokenType::Let => return,

      TokenType::While => return,
      TokenType::If => return,
      TokenType::Print => return,
      TokenType::Return => return,
      _ => parser.advance(),
    }
  }
}

fn parse_precedence(parser: &mut Parser, precedence: Precedence) {
  parser.advance();
  let token = parser.previous.unwrap().token_type;

  let prefix_rule = get_rule(token).prefix;
  if prefix_rule.is_none() {
    parser.error("Expect expression.");
    return;
  }

  let can_assign = precedence <= Precedence::Assignment;
  prefix_rule.unwrap()(parser, can_assign);

  while precedence <= get_rule(parser.current.unwrap().token_type).precedence {
    parser.advance();

    if let Some(infix_rule) = get_rule(parser.previous.unwrap().token_type).infix {
      infix_rule(parser, can_assign);
    }
  }

  if can_assign && parser.matches(TokenType::Equal) {
    parser.error("Invalid assignment target.");
  }
}

fn declaration(parser: &mut Parser) {
  if parser.matches(TokenType::Let) {
    var_declaration(parser);
  } else {
    statement(parser);
  }

  if parser.panic_mode {
    synchronize(parser);
  }
}

fn var_declaration(parser: &mut Parser) {
  parser.consume(TokenType::Identifier, "Expect variable name.");
  let variable_name = parser.previous.unwrap().get_value(&parser.scanner);

  if parser.matches(TokenType::Equal) {
    expression(parser);
  } else {
    parser.emit_opcode(OpCode::Null);
  }

  parser.consume(
    TokenType::EndOfLine,
    "Expect new line after variable declaration.",
  );

  if parser.scope_depth > 0 {
    if parser
      .locals
      .iter()
      .any(|local| local.name == variable_name && local.depth == parser.scope_depth)
    {
      parser.error(&format!(
        "Variable with name '{}' already declared in this scope.",
        variable_name
      ));
    } else {
      parser.locals.push(Local {
        name: variable_name,
        depth: parser.scope_depth,
      });
    }
  } else {
    parser.emit_opcode(OpCode::DefineGlobal);
    parser.emit_constant_string(variable_name);
  }
}

fn statement(parser: &mut Parser) {
  if parser.matches(TokenType::Print) {
    print_statement(parser);
  } else if parser.matches(TokenType::LeftBrace) {
    parser.begin_scope();
    block(parser);
    parser.end_scope();
  } else if parser.matches(TokenType::While) {
    while_statement(parser);
  } else if parser.matches(TokenType::If) {
    if_statement(parser);
  } else {
    expression_statement(parser);
  }
}

fn block(parser: &mut Parser) {
  while parser.current.unwrap().token_type != TokenType::EndOfFile
    && parser.current.unwrap().token_type != TokenType::RightBrace
  {
    declaration(parser);
  }

  parser.consume(TokenType::RightBrace, "Expect '}' after block.");
  if parser.current.unwrap().token_type == TokenType::EndOfLine {
    parser.advance();
  }
}

fn print_statement(parser: &mut Parser) {
  expression(parser);
  parser.consume(TokenType::EndOfLine, "Expect new line after value.");
  parser.emit_opcode(OpCode::Print);
}

fn if_statement(parser: &mut Parser) {
  parser.consume(TokenType::LeftParen, "Expect '(' after 'if'.");
  expression(parser);
  parser.consume(TokenType::RightParen, "Expect ')' after condition.");

  let then_jump = emit_jump(parser, OpCode::JumpIfFalse);
  parser.emit_opcode(OpCode::Pop);
  statement(parser);

  if parser.matches(TokenType::Else) {
    let else_jump = emit_jump(parser, OpCode::Jump);
    patch_jump(parser, then_jump);
    parser.emit_opcode(OpCode::Pop);
    statement(parser);
    patch_jump(parser, else_jump);
  } else {
  }
}

fn emit_jump(parser: &mut Parser, instruction: OpCode) -> usize {
  parser.emit_opcode(instruction);
  parser.emit_long_value(u16::MAX);
  parser.chunk.len() - 2
}

fn patch_jump(parser: &mut Parser, offset: usize) {
  // -2 to adjust for the bytecode for the jump offset itself
  let jump = parser.chunk.len() - offset;

  if jump > u16::MAX as usize {
    parser.error("Too much code to jump over.");
  }

  parser.chunk.code[offset] = (jump >> 8) as u8;
  parser.chunk.code[offset + 1] = jump as u8;
}

fn while_statement(parser: &mut Parser) {
  let loop_start = parser.chunk.len();
  parser.consume(TokenType::LeftParen, "Expect '(' after 'while'.");
  expression(parser);
  parser.consume(TokenType::RightParen, "Expect ')' after condition.");

  let exit_jump = emit_jump(parser, OpCode::JumpIfFalse);
  parser.emit_opcode(OpCode::Pop);
  statement(parser);

  parser.emit_opcode(OpCode::Loop);

  let offset = parser.chunk.len() - loop_start;
  if offset > u16::MAX as usize {
    parser.error("Loop body too large.");
  } else {
    parser.emit_long_value(offset as u16);
  }

  patch_jump(parser, exit_jump);
  parser.emit_opcode(OpCode::Pop);
}

fn expression_statement(parser: &mut Parser) {
  expression(parser);
  parser.consume(TokenType::EndOfLine, "Expect new line after expression.");
  parser.emit_opcode(OpCode::Pop);
}

fn expression(parser: &mut Parser) {
  parse_precedence(parser, Precedence::Assignment);
}

fn string(parser: &mut Parser, _can_assign: bool) {
  if let Some(token) = &parser.previous {
    let token_value = token.get_value(&parser.scanner);
    let value: Box<str> = token_value[1..token_value.len() - 1]
      .to_string()
      .into_boxed_str();
    parser.emit_constant(Value::String(value));
  }
}

fn number(parser: &mut Parser, _can_assign: bool) {
  if let Some(token) = &parser.previous {
    let value: f64 = token
      .get_value(&parser.scanner)
      .replace("_", "")
      .parse()
      .unwrap();
    parser.emit_constant(Value::Number(value));
  }
}

macro_rules! get {
  ($parser:expr, $index:expr, $name:expr) => {
    match $index {
      Some(index) => {
        $parser.emit_opcode(OpCode::GetLocal);
        $parser.emit_value(index as u8);
      }
      _ => {
        $parser.emit_opcode(OpCode::GetGlobal);
        $parser.emit_constant_string($name);
      }
    }
  };
}

macro_rules! set {
  ($parser:expr, $index:expr, $name:expr) => {
    match $index {
      Some(index) => {
        $parser.emit_opcode(OpCode::SetLocal);
        $parser.emit_value(index as u8);
      }
      _ => {
        $parser.emit_opcode(OpCode::SetGlobal);
        $parser.emit_constant_string($name);
      }
    }
  };
}

fn variable(parser: &mut Parser, can_assign: bool) {
  let name = parser.previous.unwrap().get_value(&parser.scanner);
  let local_index = parser.locals.iter().rposition(|local| local.name == name);

  let additional_operator = match parser.current.unwrap().token_type {
    TokenType::PlusEqual => Some(OpCode::Add),
    TokenType::MinusEqual => Some(OpCode::Subtract),
    TokenType::StarEqual => Some(OpCode::Multiply),
    TokenType::SlashEqual => Some(OpCode::Divide),
    _ => None,
  };

  if additional_operator.is_some() && can_assign {
    parser.advance();
    get!(parser, local_index, name.clone());
    expression(parser);
    parser.emit_opcode(additional_operator.unwrap());
    set!(parser, local_index, name);
  } else if parser.matches(TokenType::Equal) && can_assign {
    expression(parser);
    set!(parser, local_index, name);
  } else {
    get!(parser, local_index, name);
  }
}

fn grouping(parser: &mut Parser, _can_assign: bool) {
  expression(parser);
  parser.consume(TokenType::RightParen, "Expect ')' after expression.");
}

fn unary(parser: &mut Parser, _can_assign: bool) {
  let operator = parser.previous.unwrap().token_type;

  parse_precedence(parser, Precedence::Unary);

  match operator {
    TokenType::Minus => parser.emit_opcode(OpCode::Negate),
    TokenType::Bang => parser.emit_opcode(OpCode::Not),
    _ => {}
  }
}

fn binary(parser: &mut Parser, _can_assign: bool) {
  let operator = parser.previous.unwrap().token_type;
  let rule = get_rule(operator);
  parse_precedence(parser, get_precedence((rule.precedence as u8) + 1));

  match operator {
    TokenType::Plus => parser.emit_opcode(OpCode::Add),
    TokenType::Minus => parser.emit_opcode(OpCode::Subtract),
    TokenType::Star => parser.emit_opcode(OpCode::Multiply),
    TokenType::Slash => parser.emit_opcode(OpCode::Divide),

    TokenType::EqualEqual => parser.emit_opcode(OpCode::Equal),
    TokenType::Greater => parser.emit_opcode(OpCode::Greater),
    TokenType::Less => parser.emit_opcode(OpCode::Less),

    TokenType::BangEqual => {
      parser.emit_opcode(OpCode::Equal);
      parser.emit_opcode(OpCode::Not)
    }
    TokenType::GreaterEqual => {
      parser.emit_opcode(OpCode::Greater);
      parser.emit_opcode(OpCode::Not)
    }
    TokenType::LessEqual => {
      parser.emit_opcode(OpCode::Less);
      parser.emit_opcode(OpCode::Not)
    }

    _ => {}
  }
}

fn literal(parser: &mut Parser, _can_assign: bool) {
  match parser.previous.unwrap().token_type {
    TokenType::True => parser.emit_opcode(OpCode::True),
    TokenType::False => parser.emit_opcode(OpCode::False),
    TokenType::Null => parser.emit_opcode(OpCode::Null),
    _ => {}
  }
}

fn and(parser: &mut Parser, _can_assign: bool) {
  let jump = emit_jump(parser, OpCode::JumpIfFalse);
  parser.emit_opcode(OpCode::Pop);
  parse_precedence(parser, Precedence::And);
  patch_jump(parser, jump);
}

fn or(parser: &mut Parser, _can_assign: bool) {
  let else_jump = emit_jump(parser, OpCode::JumpIfFalse);
  let end_jump = emit_jump(parser, OpCode::Jump);

  patch_jump(parser, else_jump);
  parser.emit_opcode(OpCode::Pop);

  parse_precedence(parser, Precedence::Or);
  patch_jump(parser, end_jump);
}

fn nullish(parser: &mut Parser, _can_assign: bool) {
  let else_jump = emit_jump(parser, OpCode::JumpIfNull);
  let end_jump = emit_jump(parser, OpCode::Jump);

  patch_jump(parser, else_jump);
  parser.emit_opcode(OpCode::Pop);

  parse_precedence(parser, Precedence::Nullish);
  patch_jump(parser, end_jump);
}
