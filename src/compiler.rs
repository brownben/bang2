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

  fn emit_constant(&mut self, token: &'s Token<'s>, value: Value) {
    let constant_position = self.chunk.add_constant(value);

    if let Ok(constant_position) = u8::try_from(constant_position) {
      self.emit_opcode(token, OpCode::Constant(constant_position));
    } else if let Ok(constant_position) = u16::try_from(constant_position) {
      self.emit_opcode(token, OpCode::ConstantLong(constant_position));
    } else {
      self.error(token, Error::TooManyConstants);
    }
  }

  fn emit_constant_string(&mut self, token: &'s Token<'s>, value: &'s str) -> u8 {
    let constant_position = self.chunk.add_constant_string(value);

    if let Ok(constant_position) = u8::try_from(constant_position) {
      constant_position
    } else {
      self.error(token, Error::TooManyConstants);
      0
    }
  }

  fn emit_space(&mut self, token: &'s Token<'s>) -> usize {
    self.emit_opcode(token, OpCode::Unknown);
    self.chunk.length() - 1
  }

  fn get_jump(&mut self, token: &'s Token<'s>, instruction_location: usize) -> u16 {
    let jump = self.chunk.length() - instruction_location;

    if jump > u16::MAX as usize {
      self.error(token, Error::TooBigJump);
    }

    jump as u16
  }

  fn patch_instruction(&mut self, instruction_location: usize, code: OpCode) {
    self.chunk.patch_opcode(instruction_location, code)
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

  fn finish_chunk(&mut self, name: String) -> Chunk {
    let mut chunk = std::mem::replace(&mut self.chunk, self.chunk_stack.pop().unwrap());
    self.end_scope();
    chunk.finalize(name)
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
          let name = self.emit_constant_string(identifier, variable_name);
          self.emit_opcode(identifier, OpCode::DefineGlobal(name));
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

        let then_jump_instruction = self.emit_space(if_token);
        self.emit_opcode(if_token, OpCode::Pop);
        self.compile_statement(then);

        if let Some(otherwise) = otherwise {
          let else_token = else_token.unwrap();
          let else_jump_instruction = self.emit_space(else_token);

          let jump = self.get_jump(if_token, then_jump_instruction);
          self.patch_instruction(then_jump_instruction, OpCode::JumpIfFalse(jump));
          self.emit_opcode(else_token, OpCode::Pop);

          self.compile_statement(otherwise);
          let jump = self.get_jump(else_token, else_jump_instruction);
          self.patch_instruction(else_jump_instruction, OpCode::Jump(jump));
        } else {
          let jump = self.get_jump(if_token, then_jump_instruction);
          self.patch_instruction(then_jump_instruction, OpCode::JumpIfFalse(jump));
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

        let exit_jump = self.emit_space(token);
        self.emit_opcode(token, OpCode::Pop);

        self.compile_statement(body);

        let offset = self.length() - loop_start;
        if offset > u16::MAX as usize {
          self.error(token, Error::TooBigJump);
        } else {
          self.emit_opcode(token, OpCode::Loop(offset as u16));
        }

        let jump = self.get_jump(token, exit_jump);
        self.patch_instruction(exit_jump, OpCode::JumpIfFalse(jump));
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
          self.emit_opcode(identifier, OpCode::SetLocal(index as u8));
        } else {
          let name = self.emit_constant_string(identifier, variable_name);
          self.emit_opcode(identifier, OpCode::SetGlobal(name));
        }
      }
      Expr::Variable { token } => {
        let variable_name = token.value;
        let local_index = self
          .locals
          .iter()
          .rposition(|local| local.name == variable_name);

        if let Some(index) = local_index {
          self.emit_opcode(token, OpCode::GetLocal(index as u8));
        } else {
          let name = self.emit_constant_string(token, variable_name);
          self.emit_opcode(token, OpCode::GetGlobal(name));
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

        self.emit_opcode(token, OpCode::Call(arguments.len() as u8));
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
        let chunk = self.finish_chunk(name.unwrap_or("").to_string());

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
    let jump_instruction = self.emit_space(operator);
    self.emit_opcode(operator, OpCode::Pop);
    self.compile_expression(right);
    let jump = self.get_jump(operator, jump_instruction);
    self.patch_instruction(jump_instruction, OpCode::JumpIfFalse(jump));
  }

  fn or(&mut self, operator: &'s Token<'s>, left: &'s Expr, right: &'s Expr) {
    self.compile_expression(left);
    let else_jump = self.emit_space(operator);
    let end_jump = self.emit_space(operator);

    let jump = self.get_jump(operator, else_jump);
    self.patch_instruction(else_jump, OpCode::JumpIfFalse(jump));
    self.emit_opcode(operator, OpCode::Pop);

    self.compile_expression(right);
    let jump = self.get_jump(operator, end_jump);
    self.patch_instruction(end_jump, OpCode::Jump(jump));
  }

  fn nullish(&mut self, operator: &'s Token<'s>, left: &'s Expr, right: &'s Expr) {
    self.compile_expression(left);
    let else_jump = self.emit_space(operator);
    let end_jump = self.emit_space(operator);

    let jump = self.get_jump(operator, else_jump);
    self.patch_instruction(else_jump, OpCode::JumpIfNull(jump));
    self.emit_opcode(operator, OpCode::Pop);

    self.compile_expression(right);
    let jump = self.get_jump(operator, end_jump);
    self.patch_instruction(end_jump, OpCode::Jump(jump));
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

  let chunk = compiler.chunk.finalize("<script>".to_string());

  Ok(chunk)
}
