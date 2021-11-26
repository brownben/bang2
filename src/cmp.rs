use crate::chunk::{Chunk, OpCode};
use crate::error;
use crate::error::Error;
use crate::scanner::{Token, TokenType};
use crate::value::Value;
  use crate::ast::{Expression, Statement};
use ariadne::{Label, Report, ReportKind, Source};


#[cfg(feature = "debug-bytecode")]
use crate::chunk;
#[cfg(feature = "debug-token")]
use crate::scanner;

#[derive(Debug)]
struct Local {
  name: String,
  depth: u8,
}
struct Compiler {
  chunk: Chunk,
  ast: Vec<Statement>,
  scanner:Scanner,

  locals: Vec<Local>,
  scope_depth: u8,
}

impl Compiler {
  fn new(scanner:Scanner, ast: Vec<Statement>, from: &str) -> Self {
    Self {
      ast,
      scanner,
      chunk: Chunk::new(),

      locals: Vec::new(),
      scope_depth: 0,
    }
  }

  fn begin_scope(&mut self) {
    self.scope_depth += 1;
  }

  fn end_scope(&mut self) {
    while let Some(value) = self.locals.last() {
      if value.depth == self.scope_depth {
        self.locals.pop();
        self.emit_opcode(OpCode::Pop);
      }
    }

    self.scope_depth -= 1;
  }

  fn error_at(&mut self, token: Option<Token>, message: Error) {
    if !self.panic_mode {
      match token {
        Some(token) => {
          let diagnostic = error::get_message(message, &self.scanner, &token);
          let source: String = self.scanner.chars.iter().collect();
          let file = &self.scanner.from;

          Report::build(ReportKind::Error, file, token.start)
            .with_message(diagnostic.message)
            .with_label(Label::new((file, token.start..token.end)).with_message(diagnostic.label))
            .with_note(diagnostic.note)
            .finish()
            .eprint((file, Source::from(source)))
            .unwrap();
        }
        _ => println!("Error"),
      }
    }

    self.panic_mode = true;
    self.had_error = true;
  }

  fn end_compiler(&mut self) {
    while self.scope_depth > 0 {
      self.end_scope();
    }

    self.emit_opcode(OpCode::Return);
    self.chunk.finalize();

    #[cfg(feature = "debug-bytecode")]
    chunk::disassemble(&self.chunk, "Bytecode");
  }
}

// Emit Bytecode
impl Compiler {
  fn emit_opcode(&mut self, code: OpCode, token: Option<Token>) {
    match token {
      Some(token) => self.chunk.write(value, token.line),
      _ => self.chunk.write(value, 0),
    }
  }

  fn emit_value(&mut self, value: u8, token:Option<Token>) {
    match token {
      Some(token) => self.chunk.write_value(value, token.line),
      _ => self.chunk.write_value(value, 0),
    }
  }

  fn emit_long_value(&mut self, value: u16, token:Option<Token>) {
    match token {
      Some(token) => self.chunk.write_long_value(value, token.line),
      _ => self.chunk.write_long_value(value, 0),
    }
  }

  fn emit_constant(&mut self, value: Value, token:Token) {
    let constant_position = self.chunk.add_constant(value);

    if constant_position <= u8::max_value() as usize {
      self.emit_opcode(OpCode::Constant);
      self.emit_value(constant_position as u8);
    } else if constant_position <= u16::max_value() as usize {
      self.emit_opcode(OpCode::ConstantLong);
      self.emit_long_value(constant_position as u16);
    } else {
      self.error_at(token, Error::TooManyConstants);
    }
  }

  fn emit_constant_string(&mut self, value: String, token:Token) {
    let constant_position = self.chunk.add_constant_string(value);

    if constant_position <= u8::max_value() as usize {
      self.emit_value(constant_position as u8);
    } else {
      self.error_at(token, Error::TooManyConstants);
    }
  }

  fn emit_jump(&mut self, instruction: OpCode) -> usize {
    self.emit_opcode(instruction);
    self.emit_long_value(u16::MAX);
    self.chunk.len() - 2
  }

  fn patch_jump(&mut self, offset: usize) {
    // -2 to adjust for the bytecode for the jump offset itself
    let jump = self.chunk.len() - offset;

    if jump > u16::MAX as usize {
      self.error(Error::TooBigJump);
    }

    self.chunk.set_long_value(offset, jump as u16);
  }
}

impl Compiler {
  fn compile_statement(&mut self, statement: Statement) {
    match statement {
      Statement::Declaration {
        variable_name,
        identifier,
        expression,
        token,
      } => {
        if let Some(expression) = expression {
          self.compile_expression(expression);
        } else {
          self.emit_opcode(OpCode::Null, token);
        }

        if self.scope_depth > 0 {
          if self
            .locals
            .iter()
            .any(|local| local.name == variable_name && local.depth == self.scope_depth)
          {
            self.error_at(identifier, Error::VariableAlreadyExists);
          } else {
            self.locals.push(Local {
              name: variable_name,
              depth: self.scope_depth,
            });
          }
        } else {
          self.emit_opcode(OpCode::DefineGlobal, token);
          self.emit_constant_string(variable_name, token);
        }
      }
      Statement::If {
        condition,
        then,
        otherwise, if_token, else_token,
      } => {
        self.compile_expression(condition);

        let then_jump = self.emit_jump(OpCode::JumpIfFalse );
        self.emit_opcode(OpCode::Pop, if_token);
        self.compile_statement(*then);

        if let Some(otherwise) = otherwise {
          let else_jump = self.emit_jump(OpCode::Jump);
          self.patch_jump(then_jump);
          self.emit_opcode(OpCode::Pop, else_token);
          self.compile_statement(*otherwise);
          self.patch_jump(else_jump);
        }
      }
      Statement::While { condition, body,token } => {
        let loop_start = self.chunk.len();
        self.compile_expression(condition);

        let exit_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_opcode(OpCode::Pop);

        self.compile_statement(*body);

        self.emit_opcode(OpCode::Loop);

        let offset = self.chunk.len() - loop_start;
        if offset > u16::MAX as usize {
          self.error_at(token, Error::TooBigJump);
        } else {
          self.emit_long_value(offset as u16);
        }

        self.patch_jump(exit_jump);
        self.emit_opcode(OpCode::Pop);
      }
      Statement::Print { expression,.. } => {
        self.compile_expression(expression);
        self.emit_opcode(OpCode::Print);
      }
      Statement::Expression { expression } => {
        self.compile_expression(expression);
        self.emit_opcode(OpCode::Pop);
      }
      Statement::Block { body } => {
        self.begin_scope();
        for statement in body {
          self.compile_statement(statement);
        }
        self.end_scope();
      }
    }
  }

  fn compile_expression(&mut self, expression: Expression) {
    match expression {
      Expression::Literal {value, token} => {
        match value {
          Value::Boolean(true) => self.emit_opcode(OpCode::True),
          Value::Boolean(false) => self.emit_opcode(OpCode::False),
          Value::Null => self.emit_opcode(OpCode::Null),
          Value::Number(_) | Value::String(_) => self.emit_constant(value, token),
        };
      }
      Expression::Group { expression } => {
        self.compile_expression(*expression);
      }
      Expression::Unary {
        operator,
        expression,
      } => {
        self.compile_expression(*expression);
        match operator.token_type {
          TokenType::Minus => self.emit_opcode(OpCode::Negate),
          TokenType::Bang => self.emit_opcode(OpCode::Not),
          _ => {}
        }
      }
      Expression::Binary {
        left,
        operator,
        right,
      } => {
        self.compile_expression(*left);

        if let TokenType::And | TokenType::Or | TokenType::QuestionQuestion = operator.token_type {
          match operator.token_type {
            TokenType::And => {
              let jump = self.emit_jump(OpCode::JumpIfFalse);
              self.emit_opcode(OpCode::Pop);
              self.compile_expression(*right);
              self.patch_jump(jump);
            }
            TokenType::Or => {
              let else_jump = self.emit_jump(OpCode::JumpIfFalse);
              let end_jump = self.emit_jump(OpCode::Jump);

              self.patch_jump(else_jump);
              self.emit_opcode(OpCode::Pop);

              self.compile_expression(*right);
              self.patch_jump(end_jump);
            }
            TokenType::QuestionQuestion => {
              let else_jump = self.emit_jump(OpCode::JumpIfNull);
              let end_jump = self.emit_jump(OpCode::Jump);

              self.patch_jump(else_jump);
              self.emit_opcode(OpCode::Pop);

              self.compile_expression(*right);
              self.patch_jump(end_jump);
            }
          }
        } else {
          self.compile_expression(*right);
          match operator.token_type {
            TokenType::Plus => self.emit_opcode(OpCode::Add),
            TokenType::Minus => self.emit_opcode(OpCode::Subtract),
            TokenType::Star => self.emit_opcode(OpCode::Multiply),
            TokenType::Slash => self.emit_opcode(OpCode::Divide),

            TokenType::EqualEqual => self.emit_opcode(OpCode::Equal),
            TokenType::Greater => self.emit_opcode(OpCode::Greater),
            TokenType::Less => self.emit_opcode(OpCode::Less),

            TokenType::BangEqual => {
              self.emit_opcode(OpCode::Equal);
              self.emit_opcode(OpCode::Not);
            }
            TokenType::GreaterEqual => {
              self.emit_opcode(OpCode::Less);
              self.emit_opcode(OpCode::Not);
            }
            TokenType::LessEqual => {
              self.emit_opcode(OpCode::Greater);
              self.emit_opcode(OpCode::Not);
            }
            _ => {}
          }
        }
      }
      Expression::Assignment{identifier, expression, global} => {
        let name = identifier.get_value(&self.scanner);
        let local_index = self.locals.iter().rposition(|local| local.name == name);
        self.compile_expression(*expression);

        if let Some(local_index) = local_index {
          self.emit_opcode(OpCode::GetLocal);
          self.emit_value(local_index as u8, Some(identifier));
        } else  {
          self.emit_opcode(OpCode::GetGlobal);
          self.emit_constant_string(name, Some(identifier));
        }
      },
      Expression::Variable { identifier, global } => {
        let name = identifier.get_value(&self.scanner);
        let local_index = self.locals.iter().rposition(|local| local.name == name);

        if let Some(local_index) = local_index {
          self.emit_opcode(OpCode::GetLocal);
          self.emit_value(local_index as u8, Some(identifier));
        } else  {
          self.emit_opcode(OpCode::GetGlobal);
          self.emit_constant_string(name, Some(identifier));
        }
      }
    }
  }
}
