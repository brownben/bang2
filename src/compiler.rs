use crate::{
  ast::{Expr, Stmt},
  chunk::{Chunk, ChunkBuilder, OpCode},
  diagnostic::Diagnostic,
  tokens::{Token, TokenType},
  value::{Function, Value},
};

enum Error {
  TooBigJump,
  TooManyConstants,
  TooManyArguments,
  TooManyParameters,
  VariableAlreadyExists,
}
impl Error {
  fn get_title(&self) -> &'static str {
    match self {
      Self::TooBigJump => "Too Big Jump",
      Self::TooManyConstants => "Too Many Constants",
      Self::TooManyArguments => "Too Many Arguments",
      Self::TooManyParameters => "Too Many Parameters",
      Self::VariableAlreadyExists => "Variable Already Exists",
    }
  }

  fn get_message(&self, token: &Token) -> String {
    match self {
      Self::TooBigJump | Self::TooManyConstants => {
        "This is likely an error with the language".to_string()
      }
      Self::TooManyArguments | Self::TooManyParameters => {
        "There is a limit of 255 arguments for a function".to_string()
      }
      Self::VariableAlreadyExists => format!("Variable '{}' has been defined already", token.value),
    }
  }

  fn get_diagnostic(&self, token: &Token) -> Diagnostic {
    Diagnostic {
      title: self.get_title().to_string(),
      message: self.get_message(token),
      lines: vec![token.line],
    }
  }
}

struct Local<'s> {
  name: &'s str,
  depth: u8,
}

struct Compiler<'s> {
  chunk: ChunkBuilder,
  chunk_stack: Vec<ChunkBuilder>,

  locals: Vec<Local<'s>>,
  scope_depth: u8,

  error: Option<Diagnostic>,
}

// Emit Bytecode
impl<'s> Compiler<'s> {
  fn emit_opcode(&mut self, token: &'s Token<'s>, code: OpCode) {
    self.chunk.write_opcode(code, token.line);
  }

  fn emit_opcode_blank(&mut self, code: OpCode) {
    self.chunk.write_opcode(code, 0);
  }

  fn emit_value(&mut self, token: &'s Token<'s>, value: u8) {
    self.chunk.write_value(value, token.line);
  }

  fn emit_long_value(&mut self, token: &'s Token<'s>, value: u16) {
    self.chunk.write_long_value(value, token.line);
  }

  fn emit_constant(&mut self, token: &'s Token<'s>, value: Value) {
    let constant_position = self.chunk.add_constant(value);

    if let Ok(constant_position) = u8::try_from(constant_position) {
      self.emit_opcode(token, OpCode::Constant);
      self.emit_value(token, constant_position);
    } else if let Ok(constant_position) = u16::try_from(constant_position) {
      self.emit_opcode(token, OpCode::ConstantLong);
      self.emit_long_value(token, constant_position);
    } else {
      self.error(token, Error::TooManyConstants);
    }
  }

  fn emit_constant_string(&mut self, token: &'s Token<'s>, value: &'s str) {
    let constant_position = self.chunk.add_constant_string(value);

    if let Ok(constant_position) = u8::try_from(constant_position) {
      self.emit_value(token, constant_position);
    } else {
      self.error(token, Error::TooManyConstants);
    }
  }

  fn emit_jump(&mut self, token: &'s Token<'s>, instruction: OpCode) -> usize {
    self.emit_opcode(token, instruction);
    self.emit_long_value(token, u16::MAX);
    self.chunk.length() - 2
  }

  fn patch_jump(&mut self, token: &'s Token<'s>, offset: usize) {
    let jump = self.chunk.length() - offset;

    if jump > u16::MAX as usize {
      self.error(token, Error::TooBigJump);
    }

    self.chunk.set_long_value(offset, jump as u16);
  }

  fn length(&self) -> usize {
    self.chunk.length()
  }
}

impl<'s> Compiler<'s> {
  fn new() -> Self {
    Self {
      chunk: ChunkBuilder::new(),
      chunk_stack: Vec::new(),
      locals: Vec::new(),
      scope_depth: 0,
      error: None,
    }
  }

  fn begin_scope(&mut self) {
    self.scope_depth += 1;
  }

  fn end_scope(&mut self) {
    loop {
      if self.locals.last().is_some() && self.locals.last().unwrap().depth == self.scope_depth {
        self.locals.pop();
        self.emit_opcode_blank(OpCode::Pop);
      } else {
        break;
      }
    }

    self.scope_depth -= 1;
  }

  fn new_chunk(&mut self) {
    let chunk = std::mem::replace(&mut self.chunk, ChunkBuilder::new());
    self.chunk_stack.push(chunk);
    self.begin_scope();
  }

  fn finish_chunk(&mut self) -> Chunk {
    let mut chunk = std::mem::replace(&mut self.chunk, self.chunk_stack.pop().unwrap());
    self.end_scope();
    chunk.finalize()
  }

  fn error(&mut self, token: &'s Token<'s>, error: Error) {
    self.error = Some(error.get_diagnostic(token));
  }

  fn compile_statement(&mut self, statement: &'s Stmt) {
    match statement {
      Stmt::Declaration {
        expression,
        identifier,
        ..
      } => {
        let variable_name = identifier.value;
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
      Stmt::If {
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
        self.compile_statement(then);

        if let Some(otherwise) = otherwise {
          let else_token = else_token.unwrap();
          let else_jump = self.emit_jump(else_token, OpCode::Jump);
          self.patch_jump(if_token, then_jump);
          self.emit_opcode(else_token, OpCode::Pop);
          self.compile_statement(otherwise);
          self.patch_jump(else_token, else_jump);
        } else {
          self.patch_jump(if_token, then_jump);
          self.emit_opcode(if_token, OpCode::Pop);
        }
      }
      Stmt::While {
        token,
        condition,
        body,
        ..
      } => {
        let loop_start = self.length();
        self.compile_expression(condition);

        let exit_jump = self.emit_jump(token, OpCode::JumpIfFalse);
        self.emit_opcode(token, OpCode::Pop);

        self.compile_statement(body);
        self.emit_opcode(token, OpCode::Loop);

        let offset = self.length() - loop_start;
        if offset > u16::MAX as usize {
          self.error(token, Error::TooBigJump);
        } else {
          self.emit_long_value(token, offset as u16);
        }

        self.patch_jump(token, exit_jump);
        self.emit_opcode(token, OpCode::Pop);
      }
      Stmt::Return {
        token, expression, ..
      } => {
        if let Some(expression) = expression {
          self.compile_expression(expression);
        } else {
          self.emit_opcode(token, OpCode::Null);
        }
        self.emit_opcode(token, OpCode::Return);
      }
      Stmt::Block { body, .. } => {
        self.begin_scope();
        for statement in body {
          self.compile_statement(statement);
        }
        self.end_scope();
      }
      Stmt::Expression { expression, .. } => {
        if expression.has_side_effect() {
          self.compile_expression(expression);
          self.emit_opcode_blank(OpCode::Pop);
        }
      }
    }
  }

  fn compile_expression(&mut self, expression: &'s Expr) {
    match expression {
      Expr::Literal { token, value } => match token.ttype {
        TokenType::True => self.emit_opcode(token, OpCode::True),
        TokenType::False => self.emit_opcode(token, OpCode::False),
        TokenType::Null => self.emit_opcode(token, OpCode::Null),
        TokenType::Number => self.emit_constant(token, Value::parse_number(token.value)),
        TokenType::String => self.emit_constant(token, Value::from(*value)),
        _ => unreachable!(),
      },
      Expr::Group { expression } => {
        self.compile_expression(expression);
      }
      Expr::Unary {
        expression,
        operator,
      } => {
        self.compile_expression(expression);

        match operator.ttype {
          TokenType::Minus => self.emit_opcode(operator, OpCode::Negate),
          TokenType::Bang => self.emit_opcode(operator, OpCode::Not),
          _ => unreachable!(),
        }
      }
      Expr::Binary {
        left,
        right,
        operator,
        ..
      } => {
        match operator.ttype {
          TokenType::QuestionQuestion => return self.nullish(operator, left, right),
          TokenType::And => return self.and(operator, left, right),
          TokenType::Or => return self.or(operator, left, right),
          _ => {}
        }

        self.compile_expression(left);
        self.compile_expression(right);

        match operator.ttype {
          TokenType::Plus | TokenType::PlusEqual => self.emit_opcode(operator, OpCode::Add),
          TokenType::Minus | TokenType::MinusEqual => self.emit_opcode(operator, OpCode::Subtract),
          TokenType::Star | TokenType::StarEqual => self.emit_opcode(operator, OpCode::Multiply),
          TokenType::Slash | TokenType::SlashEqual => self.emit_opcode(operator, OpCode::Divide),
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
          _ => unreachable!(),
        }
      }
      Expr::Assignment {
        identifier,
        expression,
      } => {
        let variable_name = identifier.value;
        let local_index = self
          .locals
          .iter()
          .rposition(|local| local.name == variable_name);

        self.compile_expression(expression);

        if let Some(index) = local_index {
          self.emit_opcode(identifier, OpCode::SetLocal);
          self.emit_value(identifier, index as u8);
        } else {
          self.emit_opcode(identifier, OpCode::SetGlobal);
          self.emit_constant_string(identifier, variable_name);
        }
      }
      Expr::Variable { token } => {
        let variable_name = token.value;
        let local_index = self
          .locals
          .iter()
          .rposition(|local| local.name == variable_name);

        if let Some(index) = local_index {
          self.emit_opcode(token, OpCode::GetLocal);
          self.emit_value(token, index as u8);
        } else {
          self.emit_opcode(token, OpCode::GetGlobal);
          self.emit_constant_string(token, variable_name);
        }
      }
      Expr::Call {
        token,
        expression,
        arguments,
        ..
      } => {
        self.compile_expression(expression);

        if arguments.len() > 255 {
          self.error(token, Error::TooManyArguments);
        }

        for argument in arguments {
          self.compile_expression(argument);
        }

        self.emit_opcode(token, OpCode::Call);
        self.emit_value(token, arguments.len() as u8);
      }

      Expr::Function {
        token,
        parameters,
        body,
        name,
        ..
      } => {
        if parameters.len() > u8::MAX as usize {
          self.error(token, Error::TooManyParameters);
        };

        self.new_chunk();
        for parameter in parameters {
          self.locals.push(Local {
            name: parameter.value,
            depth: self.scope_depth,
          });
        }
        self.compile_statement(body);
        self.emit_opcode(token, OpCode::Null);
        self.emit_opcode(token, OpCode::Return);
        let chunk = self.finish_chunk();

        self.emit_constant(
          token,
          Value::from(Function {
            name: name.unwrap_or("").to_string(),
            arity: parameters.len() as u8,
            chunk,
          }),
        );
      }
    }
  }

  fn and(&mut self, operator: &'s Token<'s>, left: &'s Expr, right: &'s Expr) {
    self.compile_expression(left);
    let jump = self.emit_jump(operator, OpCode::JumpIfFalse);
    self.emit_opcode(operator, OpCode::Pop);
    self.compile_expression(right);
    self.patch_jump(operator, jump);
  }

  fn or(&mut self, operator: &'s Token<'s>, left: &'s Expr, right: &'s Expr) {
    self.compile_expression(left);
    let else_jump = self.emit_jump(operator, OpCode::JumpIfFalse);
    let end_jump = self.emit_jump(operator, OpCode::Jump);

    self.patch_jump(operator, else_jump);
    self.emit_opcode(operator, OpCode::Pop);

    self.compile_expression(right);
    self.patch_jump(operator, end_jump);
  }

  fn nullish(&mut self, operator: &'s Token<'s>, left: &'s Expr, right: &'s Expr) {
    self.compile_expression(left);
    let else_jump = self.emit_jump(operator, OpCode::JumpIfNull);
    let end_jump = self.emit_jump(operator, OpCode::Jump);

    self.patch_jump(operator, else_jump);
    self.emit_opcode(operator, OpCode::Pop);

    self.compile_expression(right);
    self.patch_jump(operator, end_jump);
  }
}

pub fn compile(ast: &[Stmt]) -> Result<Chunk, Diagnostic> {
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

  let chunk = compiler.chunk.finalize();

  Ok(chunk)
}
