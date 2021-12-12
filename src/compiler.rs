use crate::ast::{Expression, Statement};
use crate::chunk::{Chunk, OpCode};
use crate::error::{CompileError, Error};
use crate::token::{Token, TokenType};
use crate::value::Value;

#[cfg(feature = "debug-bytecode")]
use crate::chunk;

#[derive(Debug)]
struct Local {
  name: String,
  depth: u8,
}

struct Compiler {
  chunk: Chunk,

  locals: Vec<Local>,
  scope_depth: u8,

  error: Option<CompileError>,
}

// Emit Bytecode
impl Compiler {
  fn emit_opcode(&mut self, token: Token, code: OpCode) {
    self.chunk.write(code, token.line)
  }

  fn emit_opcode_blank(&mut self, code: OpCode) {
    self.chunk.write(code, 0)
  }

  fn emit_value(&mut self, token: Token, value: u8) {
    self.chunk.write_value(value, token.line)
  }

  fn emit_long_value(&mut self, token: Token, value: u16) {
    self.chunk.write_long_value(value, token.line)
  }

  fn emit_constant(&mut self, token: Token, value: Value) {
    let constant_position = self.chunk.add_constant(value);

    if constant_position <= u8::max_value() as usize {
      self.emit_opcode(token, OpCode::Constant);
      self.emit_value(token, constant_position as u8);
    } else if constant_position <= u16::max_value() as usize {
      self.emit_opcode(token, OpCode::ConstantLong);
      self.emit_long_value(token, constant_position as u16);
    } else {
      self.error(token, Error::TooManyConstants);
    }
  }

  fn emit_constant_string(&mut self, token: Token, value: String) {
    let constant_position = self.chunk.add_constant_string(value);

    if constant_position <= u8::max_value() as usize {
      self.emit_value(token, constant_position as u8);
    } else {
      self.error(token, Error::TooManyConstants);
    }
  }

  fn emit_jump(&mut self, token: Token, instruction: OpCode) -> usize {
    self.emit_opcode(token, instruction);
    self.emit_long_value(token, u16::MAX);
    self.chunk.length() - 2
  }

  fn patch_jump(&mut self, token: Token, offset: usize) {
    // -2 to adjust for the bytecode for the jump offset itself
    let jump = self.chunk.length() - offset;

    if jump > u16::MAX as usize {
      self.error(token, Error::TooBigJump);
    }

    self.chunk.set_long_value(offset, jump as u16);
  }
}

impl Compiler {
  fn new() -> Self {
    Self {
      chunk: Chunk::new(),
      locals: Vec::new(),
      scope_depth: 0,
      error: None,
    }
  }

  fn begin_scope(&mut self) {
    self.scope_depth += 1;
  }

  fn end_scope(&mut self) {
    while let Some(value) = self.locals.last() {
      if value.depth == self.scope_depth {
        self.locals.pop();
        self.emit_opcode_blank(OpCode::Pop);
      }
    }

    self.scope_depth -= 1;
  }

  fn error(&mut self, token: Token, error: Error) {
    self.error = Some(CompileError { error, token })
  }

  fn compile_statement(&mut self, statement: Statement) {
    match statement {
      Statement::Declaration {
        variable_name,
        expression,
        identifier,
        ..
      } => {
        if let Some(expression) = expression {
          self.compile_expression(expression);
        } else {
          self.emit_opcode(identifier, OpCode::Null);
        }

        if self.scope_depth > 0 {
          if self
            .locals
            .iter()
            .any(|local| local.name == variable_name && local.depth == self.scope_depth)
          {
            self.error(identifier, Error::VariableAlreadyExists);
          } else {
            self.locals.push(Local {
              name: variable_name,
              depth: self.scope_depth,
            });
          }
        } else {
          self.emit_opcode(identifier, OpCode::DefineGlobal);
          self.emit_constant_string(identifier, variable_name);
        }
      }
      Statement::If {
        if_token,
        else_token,
        condition,
        then,
        otherwise,
        ..
      } => {
        self.compile_expression(condition);

        let then_jump = self.emit_jump(if_token, OpCode::JumpIfFalse);
        self.emit_opcode(if_token, OpCode::Pop);
        self.compile_statement(*then);

        if let Some(otherwise) = otherwise {
          let else_token = else_token.unwrap();
          let else_jump = self.emit_jump(else_token, OpCode::Jump);
          self.patch_jump(if_token, then_jump);
          self.emit_opcode(else_token, OpCode::Pop);
          self.compile_statement(*otherwise);
          self.patch_jump(else_token, else_jump);
        } else {
          self.patch_jump(if_token, then_jump);
          self.emit_opcode(if_token, OpCode::Pop);
        }
      }
      Statement::While {
        token,
        condition,
        body,
        ..
      } => {
        let loop_start = self.chunk.length();
        self.compile_expression(condition);

        let exit_jump = self.emit_jump(token, OpCode::JumpIfFalse);
        self.emit_opcode(token, OpCode::Pop);

        self.compile_statement(*body);
        self.emit_opcode(token, OpCode::Loop);

        let offset = self.chunk.length() - loop_start;
        if offset > u16::MAX as usize {
          self.error(token, Error::TooBigJump);
        } else {
          self.emit_long_value(token, offset as u16);
        }

        self.patch_jump(token, exit_jump);
        self.emit_opcode(token, OpCode::Pop);
      }
      Statement::Print {
        token, expression, ..
      } => {
        self.compile_expression(expression);
        self.emit_opcode(token, OpCode::Print);
      }
      Statement::Block { body, .. } => {
        self.begin_scope();
        for statement in body {
          self.compile_statement(statement);
        }
        self.end_scope();
      }
      Statement::Expression { expression, .. } => {
        self.compile_expression(expression);
        self.emit_opcode_blank(OpCode::Pop);
      }
    }
  }

  fn compile_expression(&mut self, expression: Expression) {
    match expression {
      Expression::Literal { token, value, .. } => match value {
        Value::Boolean(true) => self.emit_opcode(token, OpCode::True),
        Value::Boolean(false) => self.emit_opcode(token, OpCode::False),
        Value::Null => self.emit_opcode(token, OpCode::Null),
        _ => self.emit_constant(token, value),
      },
      Expression::Group { expression, .. } => {
        self.compile_expression(*expression);
      }
      Expression::Unary {
        expression,
        operator,
        ..
      } => {
        self.compile_expression(*expression);

        match operator.token_type {
          TokenType::Minus => self.emit_opcode(operator, OpCode::Negate),
          TokenType::Bang => self.emit_opcode(operator, OpCode::Not),
          _ => self.error(operator, Error::UnknownUnaryOperator),
        }
      }
      Expression::Binary {
        left,
        right,
        operator,
        ..
      } => {
        match operator.token_type {
          TokenType::QuestionQuestion => return self.nullish(operator, *left, *right),
          TokenType::And => return self.and(operator, *left, *right),
          TokenType::Or => return self.or(operator, *left, *right),
          _ => {}
        }

        self.compile_expression(*left);
        self.compile_expression(*right);

        match operator.token_type {
          TokenType::Plus => self.emit_opcode(operator, OpCode::Add),
          TokenType::Minus => self.emit_opcode(operator, OpCode::Subtract),
          TokenType::Star => self.emit_opcode(operator, OpCode::Multiply),
          TokenType::Slash => self.emit_opcode(operator, OpCode::Divide),

          TokenType::EqualEqual => self.emit_opcode(operator, OpCode::Equal),
          TokenType::Greater => self.emit_opcode(operator, OpCode::Greater),
          TokenType::Less => self.emit_opcode(operator, OpCode::Less),

          TokenType::BangEqual => {
            self.emit_opcode(operator, OpCode::Equal);
            self.emit_opcode(operator, OpCode::Not);
          }
          TokenType::GreaterEqual => {
            self.emit_opcode(operator, OpCode::Less);
            self.emit_opcode(operator, OpCode::Not);
          }
          TokenType::LessEqual => {
            self.emit_opcode(operator, OpCode::Greater);
            self.emit_opcode(operator, OpCode::Not);
          }

          _ => self.error(operator, Error::UnknownBinaryOperator),
        }
      }
      Expression::Assignment {
        variable_name,
        identifier,
        expression,
        ..
      } => {
        let local_index = self
          .locals
          .iter()
          .rposition(|local| local.name == variable_name);

        self.compile_expression(*expression);

        if let Some(index) = local_index {
          self.emit_opcode(identifier, OpCode::SetLocal);
          self.emit_value(identifier, index as u8);
        } else {
          self.emit_opcode(identifier, OpCode::SetGlobal);
          self.emit_constant_string(identifier, variable_name);
        }
      }
      Expression::Variable {
        identifier,
        variable_name,
        ..
      } => {
        let local_index = self
          .locals
          .iter()
          .rposition(|local| local.name == variable_name);

        if let Some(index) = local_index {
          self.emit_opcode(identifier, OpCode::GetLocal);
          self.emit_value(identifier, index as u8);
        } else {
          self.emit_opcode(identifier, OpCode::GetGlobal);
          self.emit_constant_string(identifier, variable_name);
        }
      }
    }
  }

  fn and(&mut self, operator: Token, left: Expression, right: Expression) {
    self.compile_expression(left);
    let jump = self.emit_jump(operator, OpCode::JumpIfFalse);
    self.emit_opcode(operator, OpCode::Pop);
    self.compile_expression(right);
    self.patch_jump(operator, jump);
  }

  fn or(&mut self, operator: Token, left: Expression, right: Expression) {
    self.compile_expression(left);
    let else_jump = self.emit_jump(operator, OpCode::JumpIfFalse);
    let end_jump = self.emit_jump(operator, OpCode::Jump);

    self.patch_jump(operator, else_jump);
    self.emit_opcode(operator, OpCode::Pop);

    self.compile_expression(right);
    self.patch_jump(operator, end_jump);
  }

  fn nullish(&mut self, operator: Token, left: Expression, right: Expression) {
    self.compile_expression(left);
    let else_jump = self.emit_jump(operator, OpCode::JumpIfNull);
    let end_jump = self.emit_jump(operator, OpCode::Jump);

    self.patch_jump(operator, else_jump);
    self.emit_opcode(operator, OpCode::Pop);

    self.compile_expression(right);
    self.patch_jump(operator, end_jump);
  }
}

pub fn compile(ast: Vec<Statement>) -> Result<Chunk, CompileError> {
  let mut compiler = Compiler::new();

  for statement in ast {
    compiler.compile_statement(statement);

    if let Some(error) = compiler.error {
      return Err(error);
    }
  }

  while compiler.scope_depth > 0 {
    compiler.end_scope();
  }

  compiler.emit_opcode_blank(OpCode::Return);
  compiler.chunk.finalize();

  #[cfg(feature = "debug-bytecode")]
  chunk::disassemble(&compiler.chunk);

  Ok(compiler.chunk)
}
