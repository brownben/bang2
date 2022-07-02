use std::rc::Rc;

use crate::{
  ast::{
    expression::{
      AssignmentOperator, BinaryOperator, Expr, Expression, LiteralType, UnaryOperator,
    },
    statement::{DeclarationIdentifier, Statement, Stmt},
    Span,
  },
  builtins::get_builtin_module_value,
  chunk::{Builder as ChunkBuilder, Chunk, OpCode},
  diagnostic::Diagnostic,
  parser::parse_number,
  value::{Arity, Function, Value},
};

enum Error {
  TooBigJump,
  TooManyConstants,
  TooManyArguments,
  TooManyParameters,
  TooManyLocals,
  TooLongList,
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
      Self::TooManyLocals => "Too Many Local Variables",
      Self::TooLongList => "Too Long List",
    }
  }

  fn get_message(&self, value: &str) -> String {
    match self {
      Self::TooBigJump | Self::TooManyConstants => {
        "This is likely an error with the language".to_string()
      }
      Self::TooManyArguments => {
        "There is a limit of 255 arguments to be passed to a function".to_string()
      }
      Self::TooManyParameters => "There is a limit of 255 parameters for a function".to_string(),
      Self::TooManyLocals => "There is a limit of 255 local variables at once".to_string(),
      Self::VariableAlreadyExists => format!("Variable '{value}' has been defined already"),
      Self::BuiltinNotFound => format!("Could not find value in module '{value}'"),
      Self::TooLongList => "List is too long, can have a maximum of 2^16 elements".to_string(),
    }
  }

  fn into_diagnostic(self, value: &str, span: Span, source: &str) -> Diagnostic {
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

  locals: Vec<Local<'s>>,
  scope_depth: u8,

  chunk: ChunkBuilder,
  chunk_stack: Vec<ChunkBuilder>,
  finished_chunks: Vec<Chunk>,

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

  fn base_chunk(&mut self) -> &mut ChunkBuilder {
    if self.chunk_stack.is_empty() {
      &mut self.chunk
    } else {
      &mut self.chunk_stack[0]
    }
  }

  fn emit_constant(&mut self, span: Span, value: Value) {
    let constant_position = self.base_chunk().add_constant(value);

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
    let constant_position = self.base_chunk().add_constant_string(value);

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

    if let Ok(jump) = u16::try_from(jump) {
      self.chunk.set_long_value(offset, jump);
    } else {
      self.error(Error::TooBigJump, span, "");
    }
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
      finished_chunks: Vec::new(),
      locals: Vec::new(),
      scope_depth: 0,
      error: None,
    }
  }

  fn begin_scope(&mut self) {
    self.scope_depth += 1;
  }

  fn end_scope(&mut self) {
    while let Some(last) = self.locals.last() && last.depth == self.scope_depth {
      self.locals.pop();
      self.emit_opcode_blank(OpCode::Pop);
    }

    self.scope_depth -= 1;
  }

  fn new_chunk(&mut self) {
    let chunk = std::mem::replace(&mut self.chunk, ChunkBuilder::new());
    self.chunk_stack.push(chunk);
    self.begin_scope();
  }

  fn finish_chunk(&mut self) -> usize {
    self.end_scope();

    let chunk = std::mem::replace(&mut self.chunk, self.chunk_stack.pop().unwrap());
    let chunk_id = self.finished_chunks.len();
    self.finished_chunks.push(chunk.finalize());
    chunk_id
  }

  fn error(&mut self, error: Error, span: Span, value: &str) {
    self.error = Some(error.into_diagnostic(value, span, self.source));
  }

  fn compile_statement(&mut self, statement: &Statement<'s>) {
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

        match identifier {
          DeclarationIdentifier::Variable(identifier) => {
            self.define_variable(identifier, span);
          }
          DeclarationIdentifier::List(identifiers) => {
            identifiers
              .iter()
              .enumerate()
              .for_each(|(index, identifier)| {
                let temp_local_location = u8::try_from(self.locals.len())
                  .map_err(|_| self.error(Error::TooManyLocals, span, ""))
                  .unwrap_or(0);

                self.emit_opcode(span, OpCode::GetLocal);
                self.emit_value(span, temp_local_location);
                self.emit_constant(span, Value::from(index));
                self.emit_opcode(span, OpCode::GetIndex);
                self.define_variable(identifier, span);
              });
            self.emit_opcode(span, OpCode::Pop);
          }
        }
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
        let else_jump = self.emit_jump(span, OpCode::Jump);

        self.patch_jump(span, then_jump);
        self.emit_opcode(span, OpCode::Pop);

        if let Some(otherwise) = otherwise {
          self.compile_statement(otherwise);
        }

        self.patch_jump(span, else_jump);
      }
      Stmt::Import { module, items, .. } => {
        for item in items {
          if let Some(value) = get_builtin_module_value(module, item.name) {
            self.emit_constant(span, value);

            if let Some(alias) = item.alias {
              self.define_variable(alias, item.span);
            } else {
              self.define_variable(item.name, item.span);
            }
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

        if let Ok(offset) = u16::try_from(offset) {
          self.emit_long_value(span, offset);
        } else {
          self.error(Error::TooBigJump, span, "");
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

  fn compile_expression(&mut self, expression: &Expression<'s>) {
    let span = expression.span;

    match &expression.expr {
      Expr::Literal { type_, value } => match type_ {
        LiteralType::True => self.emit_opcode(span, OpCode::True),
        LiteralType::False => self.emit_opcode(span, OpCode::False),
        LiteralType::Null => self.emit_opcode(span, OpCode::Null),
        LiteralType::Number => self.emit_constant(span, Value::from(parse_number(value))),
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
          BinaryOperator::Plus => self.emit_opcode(span, OpCode::Add),
          BinaryOperator::Minus => self.emit_opcode(span, OpCode::Subtract),
          BinaryOperator::Multiply => self.emit_opcode(span, OpCode::Multiply),
          BinaryOperator::Divide => self.emit_opcode(span, OpCode::Divide),
          BinaryOperator::Equal => self.emit_opcode(span, OpCode::Equal),
          BinaryOperator::Greater => self.emit_opcode(span, OpCode::Greater),
          BinaryOperator::Less => self.emit_opcode(span, OpCode::Less),
          BinaryOperator::NotEqual => self.emit_opcode(span, OpCode::NotEqual),
          BinaryOperator::GreaterEqual => self.emit_opcode(span, OpCode::GreaterEqual),
          BinaryOperator::LessEqual => self.emit_opcode(span, OpCode::LessEqual),
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

        if let Some(index) = local_index && let Ok(index) = u8::try_from(index) {
          self.emit_opcode(span, OpCode::SetLocal);
          self.emit_value(span, index);
        } else if local_index.is_some() {
          self.error(Error::TooManyLocals, span, "");
        } else {
          self.emit_opcode(span, OpCode::SetGlobal);
          self.emit_constant_string(span, identifier);
        }
      }
      Expr::Variable { name } => {
        let local_index = self.locals.iter().rposition(|local| local.name == *name);

        if let Some(index) = local_index && let Ok(index) = u8::try_from(index) {
          self.emit_opcode(span, OpCode::GetLocal);
          self.emit_value(span, index);
        } else if local_index.is_some() {
          self.error(Error::TooManyLocals, span, "");
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

        let arguments_length = u8::try_from(arguments.len()).unwrap_or_else(|_| {
          self.error(Error::TooManyArguments, span, "");
          255
        });

        for argument in arguments {
          self.compile_expression(argument);
        }

        self.emit_opcode(span, OpCode::Call);
        self.emit_value(span, arguments_length);
      }

      Expr::Function {
        parameters,
        body,
        name,
        ..
      } => {
        let arity = u8::try_from(parameters.len()).unwrap_or_else(|_| {
          self.error(Error::TooManyParameters, span, "");
          255
        });

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
        let chunk = self.finish_chunk();

        self.emit_constant(
          span,
          Value::from(Function {
            name: name.unwrap_or("").to_string(),
            arity: Arity::new(
              arity,
              parameters.iter().any(|parameter| parameter.catch_remaining),
            ),
            start: chunk,
          }),
        );
      }
      Expr::Comment { expression, .. } => self.compile_expression(expression),
      Expr::List { items } => {
        for item in items {
          self.compile_expression(item);
        }

        if let Ok(length) = u8::try_from(items.len()) {
          self.emit_opcode(span, OpCode::List);
          self.emit_value(span, length);
        } else if let Ok(length) = u16::try_from(items.len()) {
          self.emit_opcode(span, OpCode::ListLong);
          self.emit_long_value(span, length);
        } else {
          self.error(Error::TooLongList, span, "");
        }
      }
      Expr::Index { expression, index } => {
        self.compile_expression(expression);
        self.compile_expression(index);
        self.emit_opcode(span, OpCode::GetIndex);
      }
      Expr::IndexAssignment {
        expression,
        index,
        value,
        assignment_operator,
      } => {
        self.begin_scope();

        // Calculate the index and expression once before they are assigned
        self.compile_expression(expression);
        let expression_variable = self.define_variable("$index_assignment_expr$", span);
        self.compile_expression(index);
        let index_variable = self.define_variable("$index_assignment_index$", span);

        // Calculate the value to assign
        if let Some(operator) = *assignment_operator {
          self.get_local(expression_variable, span);
          self.get_local(index_variable, span);
          self.emit_opcode(span, OpCode::GetIndex);
          self.compile_expression(value);
          match operator {
            AssignmentOperator::Plus => self.emit_opcode(span, OpCode::Add),
            AssignmentOperator::Minus => self.emit_opcode(span, OpCode::Subtract),
            AssignmentOperator::Multiply => self.emit_opcode(span, OpCode::Multiply),
            AssignmentOperator::Divide => self.emit_opcode(span, OpCode::Divide),
          }
        } else {
          self.compile_expression(value);
        }

        // Set the index
        self.get_local(expression_variable, span);
        self.get_local(index_variable, span);
        self.emit_opcode(span, OpCode::SetIndex);

        self.end_scope();
      }
    }
  }

  fn get_local(&mut self, index: usize, span: Span) {
    if let Ok(index) = u8::try_from(index) {
      self.emit_opcode(span, OpCode::GetLocal);
      self.emit_value(span, index);
    } else {
      self.error(Error::TooManyLocals, span, "");
    }
  }

  fn define_variable(&mut self, identifier: &'s str, span: Span) -> usize {
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
        });
      }
    } else {
      self.emit_opcode(span, OpCode::DefineGlobal);
      self.emit_constant_string(span, identifier);
    }

    self.locals.len().saturating_sub(1)
  }

  fn and(&mut self, span: Span, left: &Expression<'s>, right: &Expression<'s>) {
    self.compile_expression(left);
    let jump = self.emit_jump(span, OpCode::JumpIfFalse);
    self.emit_opcode(span, OpCode::Pop);
    self.compile_expression(right);
    self.patch_jump(span, jump);
  }

  fn or(&mut self, span: Span, left: &Expression<'s>, right: &Expression<'s>) {
    self.compile_expression(left);
    let else_jump = self.emit_jump(span, OpCode::JumpIfFalse);
    let end_jump = self.emit_jump(span, OpCode::Jump);

    self.patch_jump(span, else_jump);
    self.emit_opcode(span, OpCode::Pop);

    self.compile_expression(right);
    self.patch_jump(span, end_jump);
  }

  fn nullish(&mut self, span: Span, left: &Expression<'s>, right: &Expression<'s>) {
    self.compile_expression(left);
    let else_jump = self.emit_jump(span, OpCode::JumpIfNull);
    let end_jump = self.emit_jump(span, OpCode::Jump);

    self.patch_jump(span, else_jump);
    self.emit_opcode(span, OpCode::Pop);

    self.compile_expression(right);
    self.patch_jump(span, end_jump);
  }

  fn pipeline(&mut self, span: Span, left: &Expression<'s>, right: &Expression<'s>) {
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

      let arguments_length = if let Ok(length) = u8::try_from(arguments.len()) && length < 255 {
        length + 1
      } else {
        self.error(Error::TooManyArguments, span, "");
        255
      };

      self.compile_expression(left);
      for argument in arguments {
        self.compile_expression(argument);
      }

      self.emit_opcode(span, OpCode::Call);
      self.emit_value(span, arguments_length);
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

  compiler.emit_opcode_blank(OpCode::Return);

  let mut chunk = compiler.chunk.finalize();
  let chunk_locations: Vec<_> = compiler
    .finished_chunks
    .iter()
    .map(|c| chunk.merge(c))
    .collect();

  for constant in &mut chunk.constants {
    if let Value::Function(func) = constant {
      Rc::get_mut(func).unwrap().start = chunk_locations[func.start];
    };
  }

  Ok(chunk)
}
