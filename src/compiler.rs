use crate::{
  ast::{BinaryOperator, Expr, Expression, LiteralType, Span, Statement, Stmt, UnaryOperator},
  builtins::get_builtin_module_value,
  chunk::{Chunk, ChunkBuilder, OpCode},
  diagnostic::Diagnostic,
  value::{Function, Value},
};

enum Error {
  TooBigJump,
  TooManyConstants,
  TooManyArguments,
  TooManyParameters,
  VariableAlreadyExists,
  BuiltinNotFound,
}
impl Error {
  fn get_title(&self) -> &'static str {
    match self {
      Self::TooBigJump => "Too Big Jump",
      Self::TooManyConstants => "Too Many Constants",
      Self::TooManyArguments => "Too Many Arguments",
      Self::TooManyParameters => "Too Many Parameters",
      Self::VariableAlreadyExists => "Variable Already Exists",
      Self::BuiltinNotFound => "Builtin Not Found",
    }
  }

  fn get_message(&self, value: &str) -> String {
    match self {
      Self::TooBigJump | Self::TooManyConstants => {
        "This is likely an error with the language".to_string()
      }
      Self::TooManyArguments | Self::TooManyParameters => {
        "There is a limit of 255 arguments for a function".to_string()
      }
      Self::VariableAlreadyExists => format!("Variable '{value}' has been defined already"),
      Self::BuiltinNotFound => format!("Could not find value in module '{value}'"),
    }
  }

  fn get_diagnostic(&self, value: &str, span: Span, source: &str) -> Diagnostic {
    Diagnostic {
      title: self.get_title().to_string(),
      message: self.get_message(value),
      lines: vec![span.get_line_number(source)],
    }
  }
}

struct Local<'s> {
  name: &'s str,
  depth: u8,
}

struct Compiler<'s> {
  source: &'s str,

  chunk: ChunkBuilder,
  chunk_stack: Vec<ChunkBuilder>,

  locals: Vec<Local<'s>>,
  scope_depth: u8,

  error: Option<Diagnostic>,
}

// Emit Bytecode
impl<'s> Compiler<'s> {
  fn emit_opcode(&mut self, span: Span, code: OpCode) {
    self
      .chunk
      .write_opcode(code, span.get_line_number(self.source));
  }

  fn emit_opcode_blank(&mut self, code: OpCode) {
    self.chunk.write_opcode(code, 0);
  }

  fn emit_value(&mut self, span: Span, value: u8) {
    self
      .chunk
      .write_value(value, span.get_line_number(self.source));
  }

  fn emit_long_value(&mut self, span: Span, value: u16) {
    self
      .chunk
      .write_long_value(value, span.get_line_number(self.source));
  }

  fn emit_constant(&mut self, span: Span, value: Value) {
    let constant_position = self.chunk.add_constant(value);

    if let Ok(constant_position) = u8::try_from(constant_position) {
      self.emit_opcode(span, OpCode::Constant);
      self.emit_value(span, constant_position);
    } else if let Ok(constant_position) = u16::try_from(constant_position) {
      self.emit_opcode(span, OpCode::ConstantLong);
      self.emit_long_value(span, constant_position);
    } else {
      self.error(Error::TooManyConstants, span, "");
    }
  }

  fn emit_constant_string(&mut self, span: Span, value: &'s str) {
    let constant_position = self.chunk.add_constant_string(value);

    if let Ok(constant_position) = u8::try_from(constant_position) {
      self.emit_value(span, constant_position);
    } else {
      self.error(Error::TooManyConstants, span, "");
    }
  }

  fn emit_jump(&mut self, span: Span, instruction: OpCode) -> usize {
    self.emit_opcode(span, instruction);
    self.emit_long_value(span, u16::MAX);
    self.chunk.length() - 2
  }

  fn patch_jump(&mut self, span: Span, offset: usize) {
    let jump = self.chunk.length() - offset;

    if jump > u16::MAX as usize {
      self.error(Error::TooBigJump, span, "");
    }

    self.chunk.set_long_value(offset, jump as u16);
  }

  fn length(&self) -> usize {
    self.chunk.length()
  }
}

impl<'s> Compiler<'s> {
  fn new(source: &'s str) -> Self {
    Self {
      source,
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

  fn error(&mut self, error: Error, span: Span, value: &str) {
    self.error = Some(error.get_diagnostic(value, span, self.source));
  }

  fn compile_statement(&mut self, statement: &'s Statement) {
    let span = statement.span;

    match &statement.stmt {
      Stmt::Declaration {
        expression,
        identifier,
        ..
      } => {
        if let Some(expression) = expression {
          self.compile_expression(expression);
        } else {
          self.emit_opcode(span, OpCode::Null);
        }

        self.define_variable(identifier, span);
      }
      Stmt::If {
        condition,
        then,
        otherwise,
        ..
      } => {
        self.compile_expression(condition);

        let then_jump = self.emit_jump(span, OpCode::JumpIfFalse);
        self.emit_opcode(span, OpCode::Pop);
        self.compile_statement(then);

        if let Some(otherwise) = otherwise {
          let else_jump = self.emit_jump(span, OpCode::Jump);
          self.patch_jump(span, then_jump);
          self.emit_opcode(span, OpCode::Pop);
          self.compile_statement(otherwise);
          self.patch_jump(span, else_jump);
        } else {
          self.patch_jump(span, then_jump);
          self.emit_opcode(span, OpCode::Pop);
        }
      }
      Stmt::Import { module, items, .. } => {
        for item in items {
          if let Some(value) = get_builtin_module_value(module, item.name) {
            self.emit_constant(span, value);
            self.define_variable(item.name, item.span);
          } else {
            self.error(Error::BuiltinNotFound, item.span, module);
          }
        }
      }
      Stmt::While {
        condition, body, ..
      } => {
        let loop_start = self.length();
        self.compile_expression(condition);

        let exit_jump = self.emit_jump(span, OpCode::JumpIfFalse);
        self.emit_opcode(span, OpCode::Pop);

        self.compile_statement(body);
        self.emit_opcode(span, OpCode::Loop);

        let offset = self.length() - loop_start;
        if offset > u16::MAX as usize {
          self.error(Error::TooBigJump, span, "");
        } else {
          self.emit_long_value(span, offset as u16);
        }

        self.patch_jump(span, exit_jump);
        self.emit_opcode(span, OpCode::Pop);
      }
      Stmt::Return { expression, .. } => {
        if let Some(expression) = expression {
          self.compile_expression(expression);
        } else {
          self.emit_opcode(span, OpCode::Null);
        }
        self.emit_opcode(span, OpCode::Return);
      }
      Stmt::Block { body, .. } => {
        self.begin_scope();
        for statement in body {
          self.compile_statement(statement);
        }
        self.end_scope();
      }
      Stmt::Expression { expression, .. } => {
        self.compile_expression(expression);
        self.emit_opcode_blank(OpCode::Pop);
      }
      Stmt::Comment { .. } => {}
    }
  }

  fn compile_expression(&mut self, expression: &'s Expression) {
    let span = expression.span;

    match &expression.expr {
      Expr::Literal { type_, value } => match type_ {
        LiteralType::True => self.emit_opcode(span, OpCode::True),
        LiteralType::False => self.emit_opcode(span, OpCode::False),
        LiteralType::Null => self.emit_opcode(span, OpCode::Null),
        LiteralType::Number => self.emit_constant(span, Value::parse_number(value)),
        LiteralType::String => self.emit_constant(span, Value::from(*value)),
      },
      Expr::Group { expression, .. } => {
        self.compile_expression(expression);
      }
      Expr::Unary {
        expression,
        operator,
      } => {
        self.compile_expression(expression);

        match operator {
          UnaryOperator::Minus => self.emit_opcode(span, OpCode::Negate),
          UnaryOperator::Not => self.emit_opcode(span, OpCode::Not),
        }
      }
      Expr::Binary {
        left,
        right,
        operator,
        ..
      } => {
        match operator {
          BinaryOperator::Nullish => return self.nullish(span, left, right),
          BinaryOperator::And => return self.and(span, left, right),
          BinaryOperator::Or => return self.or(span, left, right),
          BinaryOperator::Pipeline => return self.pipeline(span, left, right),
          _ => {}
        }

        self.compile_expression(left);
        self.compile_expression(right);

        match operator {
          BinaryOperator::Plus | BinaryOperator::PlusEqual => self.emit_opcode(span, OpCode::Add),
          BinaryOperator::Minus | BinaryOperator::MinusEqual => {
            self.emit_opcode(span, OpCode::Subtract)
          }
          BinaryOperator::Multiply | BinaryOperator::MultiplyEqual => {
            self.emit_opcode(span, OpCode::Multiply)
          }
          BinaryOperator::Divide | BinaryOperator::DivideEqual => {
            self.emit_opcode(span, OpCode::Divide)
          }
          BinaryOperator::Equal => self.emit_opcode(span, OpCode::Equal),
          BinaryOperator::Greater => self.emit_opcode(span, OpCode::Greater),
          BinaryOperator::Less => self.emit_opcode(span, OpCode::Less),
          BinaryOperator::NotEqual => {
            self.emit_opcode(span, OpCode::Equal);
            self.emit_opcode(span, OpCode::Not);
          }
          BinaryOperator::GreaterEqual => {
            self.emit_opcode(span, OpCode::Less);
            self.emit_opcode(span, OpCode::Not);
          }
          BinaryOperator::LessEqual => {
            self.emit_opcode(span, OpCode::Greater);
            self.emit_opcode(span, OpCode::Not);
          }
          _ => unreachable!(),
        }
      }
      Expr::Assignment {
        identifier,
        expression,
      } => {
        let local_index = self
          .locals
          .iter()
          .rposition(|local| local.name == *identifier);

        self.compile_expression(expression);

        if let Some(index) = local_index {
          self.emit_opcode(span, OpCode::SetLocal);
          self.emit_value(span, index as u8);
        } else {
          self.emit_opcode(span, OpCode::SetGlobal);
          self.emit_constant_string(span, identifier);
        }
      }
      Expr::Variable { name } => {
        let local_index = self.locals.iter().rposition(|local| local.name == *name);

        if let Some(index) = local_index {
          self.emit_opcode(span, OpCode::GetLocal);
          self.emit_value(span, index as u8);
        } else {
          self.emit_opcode(span, OpCode::GetGlobal);
          self.emit_constant_string(span, name);
        }
      }
      Expr::Call {
        expression,
        arguments,
        ..
      } => {
        self.compile_expression(expression);

        if arguments.len() > 255 {
          self.error(Error::TooManyArguments, span, "");
        }

        for argument in arguments {
          self.compile_expression(argument);
        }

        self.emit_opcode(span, OpCode::Call);
        self.emit_value(span, arguments.len() as u8);
      }

      Expr::Function {
        parameters,
        body,
        name,
        ..
      } => {
        if parameters.len() > u8::MAX as usize {
          self.error(Error::TooManyParameters, span, "");
        };

        self.new_chunk();
        for parameter in parameters {
          self.locals.push(Local {
            name: parameter.name,
            depth: self.scope_depth,
          });
        }
        self.compile_statement(body);
        self.emit_opcode(span, OpCode::Null);
        self.emit_opcode(span, OpCode::Return);
        let chunk = self.finish_chunk(name.unwrap_or("").to_string());

        self.emit_constant(
          span,
          Value::from(Function {
            name: name.unwrap_or("").to_string(),
            arity: parameters.len() as u8,
            chunk,
          }),
        );
      }
      Expr::Comment { expression, .. } => self.compile_expression(expression),
    }
  }

  fn define_variable(&mut self, identifier: &'s str, span: Span) {
    if self.scope_depth > 0 {
      if self
        .locals
        .iter()
        .any(|local| local.name == identifier && local.depth == self.scope_depth)
      {
        self.error(Error::VariableAlreadyExists, span, identifier);
      } else {
        self.locals.push(Local {
          name: identifier,
          depth: self.scope_depth,
        })
      }
    } else {
      self.emit_opcode(span, OpCode::DefineGlobal);
      self.emit_constant_string(span, identifier);
    }
  }
  fn and(&mut self, span: Span, left: &'s Expression, right: &'s Expression) {
    self.compile_expression(left);
    let jump = self.emit_jump(span, OpCode::JumpIfFalse);
    self.emit_opcode(span, OpCode::Pop);
    self.compile_expression(right);
    self.patch_jump(span, jump);
  }

  fn or(&mut self, span: Span, left: &'s Expression, right: &'s Expression) {
    self.compile_expression(left);
    let else_jump = self.emit_jump(span, OpCode::JumpIfFalse);
    let end_jump = self.emit_jump(span, OpCode::Jump);

    self.patch_jump(span, else_jump);
    self.emit_opcode(span, OpCode::Pop);

    self.compile_expression(right);
    self.patch_jump(span, end_jump);
  }

  fn nullish(&mut self, span: Span, left: &'s Expression, right: &'s Expression) {
    self.compile_expression(left);
    let else_jump = self.emit_jump(span, OpCode::JumpIfNull);
    let end_jump = self.emit_jump(span, OpCode::Jump);

    self.patch_jump(span, else_jump);
    self.emit_opcode(span, OpCode::Pop);

    self.compile_expression(right);
    self.patch_jump(span, end_jump);
  }

  fn pipeline(&mut self, span: Span, left: &'s Expression, right: &'s Expression) {
    let mut right = right;

    // If right is a comment, unwrap it
    if let Expr::Comment { expression, .. } = &right.expr {
      right = expression;
    }

    if let Expr::Call {
      expression,
      arguments,
      ..
    } = &right.expr
    {
      self.compile_expression(expression);

      if arguments.len() > 254 {
        self.error(Error::TooManyArguments, span, "");
      }

      self.compile_expression(left);
      for argument in arguments {
        self.compile_expression(argument);
      }

      self.emit_opcode(span, OpCode::Call);
      self.emit_value(span, arguments.len() as u8 + 1);
    } else {
      self.compile_expression(right);
      self.compile_expression(left);
      self.emit_opcode(span, OpCode::Call);
      self.emit_value(span, 1);
    }
  }
}

pub fn compile(source: &str, ast: &[Statement]) -> Result<Chunk, Diagnostic> {
  let mut compiler = Compiler::new(source);

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
