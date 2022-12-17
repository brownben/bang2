use crate::{
  chunk::{Chunk, OpCode},
  collections::HashMap,
  context::{BytecodeFunctionCreator, Context, ImportValue},
  value::{Arity, ClosureKind, Function, Value},
};
use bang_syntax::{
  ast::{
    expression::{operators, Expr, Expression, LiteralType},
    statement::{DeclarationIdentifier, Statement, Stmt},
  },
  Diagnostic, Parser, Span,
};
use smallvec::SmallVec;
use std::{mem, rc::Rc};

enum Error {
  TooBigJump,
  TooManyConstants,
  TooManyArguments,
  TooManyParameters,
  TooManyLocals,
  TooLongList,
  TooLargeDict,
  VariableAlreadyExists,
  ModuleNotFound,
  ItemNotFound,
}
impl Error {
  fn get_title(&self) -> &'static str {
    match self {
      Self::TooBigJump => "Too Big Jump",
      Self::TooManyConstants => "Too Many Constants",
      Self::TooManyArguments => "Too Many Arguments",
      Self::TooManyParameters => "Too Many Parameters",
      Self::VariableAlreadyExists => "Variable Already Exists",
      Self::ModuleNotFound => "Module Not Found",
      Self::ItemNotFound => "Item Not Found in Module",
      Self::TooManyLocals => "Too Many Local Variables",
      Self::TooLongList => "Too Long List",
      Self::TooLargeDict => "Too Large Dict",
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
      Self::ModuleNotFound => format!("Could not find module '{value}'"),
      Self::ItemNotFound => format!("Could not find '{value}' in module"),
      Self::TooLongList => "List is too long, can have a maximum of 2^16 elements".to_string(),
      Self::TooLargeDict => {
        "Dictionary is too large, can have a maximum of 255 static items".to_string()
      }
    }
  }

  fn into_diagnostic(self, value: &str, span: Span, source: &str) -> Diagnostic {
    Diagnostic {
      title: self.get_title().to_string(),
      message: self.get_message(value),
      line: span.get_line_number(source),
      span,
    }
  }
}

struct Local<'s> {
  name: &'s str,
  depth: u8,
  closed: bool,
}

#[derive(Default)]
struct Compiler<'s, 'c> {
  source: &'s str,
  context: &'c dyn Context,

  locals: Vec<Vec<Local<'s>>>,
  closures: Vec<SmallVec<[(u8, ClosureKind); 8]>>,
  scope_depth: u8,

  chunk: Chunk,
  chunk_stack: Vec<Chunk>,

  import_cache: HashMap<(&'s str, &'s str), Value>,

  error: Option<Diagnostic>,
}

// Emit Bytecode
impl Compiler<'_, '_> {
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

  fn emit_constant(&mut self, span: Span, value: Value) -> usize {
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

    constant_position
  }

  fn emit_constant_string(&mut self, span: Span, value: &str) {
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

    if let Ok(jump) = u16::try_from(jump) {
      self.chunk.set_long_value(offset, jump);
    } else {
      self.error(Error::TooBigJump, span, "");
    }
  }

  fn emit_local_index(&mut self, index: usize, span: Span) {
    if let Ok(index) = u8::try_from(index) {
      self.emit_value(span, index);
    } else {
      self.error(Error::TooManyLocals, span, "");
    }
  }

  fn length(&self) -> usize {
    self.chunk.length()
  }
}

impl<'s, 'c> Compiler<'s, 'c> {
  fn new(source: &'s str, context: &'c dyn Context) -> Self {
    Self {
      source,
      context,

      locals: vec![Vec::new()],

      ..Default::default()
    }
  }

  fn finish(mut self) -> Chunk {
    self.emit_opcode_blank(OpCode::Null);
    self.emit_opcode_blank(OpCode::Return);

    self.chunk.finalize()
  }

  fn begin_scope(&mut self) {
    self.scope_depth += 1;
  }

  fn end_scope(&mut self) {
    let locals = self.locals.last_mut().expect("Local stack to have item");
    let mut count = 0;

    while let Some(last) = locals.last() && last.depth == self.scope_depth {
      locals.pop();
      count += 1;
    }

    (0..count).for_each(|_| self.emit_opcode_blank(OpCode::Pop));
    self.scope_depth -= 1;
  }

  fn new_chunk(&mut self) {
    let chunk = mem::replace(&mut self.chunk, Chunk::new());
    self.chunk_stack.push(chunk);
    self.locals.push(Vec::new());
    self.begin_scope();
  }

  fn finish_chunk(&mut self) -> Chunk {
    self.end_scope();
    self.locals.pop();

    let chunk = mem::replace(&mut self.chunk, self.chunk_stack.pop().unwrap());
    chunk.finalize()
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
          DeclarationIdentifier::Ordered(identifiers) => {
            identifiers
              .iter()
              .enumerate()
              .for_each(|(index, identifier)| {
                let locals = self.locals.last().expect("Local stack to have item");
                let temp_local_location = u8::try_from(locals.len())
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
          DeclarationIdentifier::Named(identifiers) => {
            for identifier in identifiers {
              let locals = self.locals.last().expect("Local stack to have item");
              let temp_local_location = u8::try_from(locals.len())
                .map_err(|_| self.error(Error::TooManyLocals, span, ""))
                .unwrap_or(0);

              self.emit_opcode(span, OpCode::GetLocal);
              self.emit_value(span, temp_local_location);
              self.emit_constant(span, Value::from(identifier.name));
              self.emit_opcode(span, OpCode::GetIndex);
              self.define_variable(identifier.get_name(), span);
            }
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
          self.import(module, item.name, item.span);
          self.define_variable(item.get_name(), item.span);
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
        LiteralType::Number => {
          self.emit_constant(span, Value::from(Parser::number(value)));
        }
        LiteralType::String => {
          self.emit_constant(span, Value::from(*value));
        }
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
          operators::Unary::Minus => self.emit_opcode(span, OpCode::Negate),
          operators::Unary::Not => self.emit_opcode(span, OpCode::Not),
        }
      }
      Expr::Binary {
        left,
        right,
        operator,
        ..
      } => {
        match operator {
          operators::Binary::Nullish => return self.nullish(span, left, right),
          operators::Binary::And => return self.and(span, left, right),
          operators::Binary::Or => return self.or(span, left, right),
          operators::Binary::Pipeline => return self.pipeline(span, left, right),
          _ => {}
        }

        self.compile_expression(left);
        self.compile_expression(right);

        match operator {
          operators::Binary::Plus => self.emit_opcode(span, OpCode::Add),
          operators::Binary::Minus => self.emit_opcode(span, OpCode::Subtract),
          operators::Binary::Multiply => self.emit_opcode(span, OpCode::Multiply),
          operators::Binary::Divide => self.emit_opcode(span, OpCode::Divide),
          operators::Binary::Equal => self.emit_opcode(span, OpCode::Equal),
          operators::Binary::Greater => self.emit_opcode(span, OpCode::Greater),
          operators::Binary::Less => self.emit_opcode(span, OpCode::Less),
          operators::Binary::NotEqual => self.emit_opcode(span, OpCode::NotEqual),
          operators::Binary::GreaterEqual => self.emit_opcode(span, OpCode::GreaterEqual),
          operators::Binary::LessEqual => self.emit_opcode(span, OpCode::LessEqual),
          _ => unreachable!(),
        }
      }
      Expr::Assignment {
        identifier,
        expression,
      } => {
        self.compile_expression(expression);

        let locals = self.locals.last().expect("Local stack to have item");
        if let Some(index) = locals.iter().rposition(|local| local.name == *identifier) {
          let local = &locals[index];

          if local.closed {
            self.emit_opcode(span, OpCode::SetAllocated);
          } else {
            self.emit_opcode(span, OpCode::SetLocal);
          }
          self.emit_local_index(index, span);

          return;
        }

        if let Some((scope_index, local_index)) = self
          .locals
          .iter()
          .enumerate()
          .rev()
          .skip(1)
          .find_map(|(scope_index, locals)| {
            locals
              .iter()
              .rposition(|local| local.name == *identifier)
              .map(|local_index| (scope_index, local_index))
          })
        {
          let closure_kind =
            mem::replace(&mut self.locals[scope_index][local_index].closed, true).into();

          let (upvalue_index, _) = self.closures.iter_mut().skip(scope_index).fold(
            (local_index, closure_kind),
            |(index, closure_kind), closures| {
              let index = closures
                .iter()
                .rposition(|(i, _)| usize::from(*i) == index)
                .unwrap_or_else(|| {
                  closures.push((u8::try_from(index).unwrap_or(0), closure_kind));
                  closures.len() - 1
                });
              (index, ClosureKind::Upvalue)
            },
          );

          self.emit_opcode(span, OpCode::SetUpvalue);
          self.emit_local_index(upvalue_index, span);

          return;
        }

        self.emit_opcode(span, OpCode::SetGlobal);
        self.emit_constant_string(span, identifier);
      }
      Expr::Variable { name } => {
        let locals = self.locals.last().expect("Local stack to have item");
        if let Some(index) = locals.iter().rposition(|local| local.name == *name) {
          let local = &locals[index];

          if local.closed {
            self.emit_opcode(span, OpCode::GetAllocated);
          } else {
            self.emit_opcode(span, OpCode::GetLocal);
          }
          self.emit_local_index(index, span);

          return;
        }

        if let Some((scope_index, local_index)) = self
          .locals
          .iter()
          .enumerate()
          .rev()
          .skip(1)
          .find_map(|(scope_index, locals)| {
            locals
              .iter()
              .rposition(|local| local.name == *name)
              .map(|local_index| (scope_index, local_index))
          })
        {
          let closure_kind =
            mem::replace(&mut self.locals[scope_index][local_index].closed, true).into();

          let (upvalue_index, _) = self.closures.iter_mut().skip(scope_index).fold(
            (local_index, closure_kind),
            |(index, closure_kind), closures| {
              let index = closures
                .iter()
                .rposition(|(i, _)| usize::from(*i) == index)
                .unwrap_or_else(|| {
                  closures.push((u8::try_from(index).unwrap_or(0), closure_kind));
                  closures.len() - 1
                });
              (index, ClosureKind::Upvalue)
            },
          );

          self.emit_opcode(span, OpCode::GetUpvalue);
          self.emit_local_index(upvalue_index, span);

          return;
        }

        self.emit_opcode(span, OpCode::GetGlobal);
        self.emit_constant_string(span, name);
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

        self.closures.push(SmallVec::new());
        self.new_chunk();
        for parameter in parameters {
          self.define_variable(parameter.name, parameter.span);
        }
        self.compile_statement(body);
        self.emit_opcode(span, OpCode::Null);
        self.emit_opcode(span, OpCode::Return);
        let chunk = self.finish_chunk();

        let upvalues = self.closures.pop().expect("Closure stack to have item");
        let has_closure = !upvalues.is_empty();

        self.emit_constant(
          span,
          Value::from(Function {
            name: name.unwrap_or("").into(),
            arity: Arity::new(
              arity,
              parameters.iter().any(|parameter| parameter.catch_remaining),
            ),
            chunk: chunk.into(),
            upvalues,
          }),
        );

        if has_closure {
          self.emit_opcode(span, OpCode::Closure);
        }
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
      Expr::Dictionary { items } => {
        for (key, value) in items {
          self.compile_expression(key);
          self.compile_expression(value);
        }

        if let Ok(length) = u8::try_from(items.len()) {
          self.emit_opcode(span, OpCode::Dict);
          self.emit_value(span, length);
        } else {
          self.error(Error::TooLargeDict, span, "");
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
        self.compile_expression(expression);
        self.compile_expression(index);

        if let Some(operator) = *assignment_operator {
          self.emit_opcode(span, OpCode::GetTemp);
          self.emit_value(span, 1);
          self.emit_opcode(span, OpCode::GetTemp);
          self.emit_value(span, 1);

          self.emit_opcode(span, OpCode::GetIndex);
          self.compile_expression(value);
          match operator {
            operators::Assignment::Plus => self.emit_opcode(span, OpCode::Add),
            operators::Assignment::Minus => self.emit_opcode(span, OpCode::Subtract),
            operators::Assignment::Multiply => self.emit_opcode(span, OpCode::Multiply),
            operators::Assignment::Divide => self.emit_opcode(span, OpCode::Divide),
          }
        } else {
          self.compile_expression(value);
        }

        self.emit_opcode(span, OpCode::SetIndex);
      }
      Expr::FormatString {
        expressions,
        strings,
      } => {
        self.emit_constant(span, strings[0].clone().into());
        for (index, expression) in expressions.iter().enumerate() {
          self.compile_expression(expression);
          self.emit_opcode(span, OpCode::ToString);
          self.emit_opcode(span, OpCode::Add);
          self.emit_constant(span, strings[index + 1].clone().into());
          self.emit_opcode(span, OpCode::Add);
        }
      }
      Expr::ModuleAccess { module, item } => self.import(module, item, span),
    }
  }

  fn define_variable(&mut self, identifier: &'s str, span: Span) -> usize {
    if self.scope_depth > 0 {
      let locals = self.locals.last_mut().expect("Local stack to have item");

      if locals
        .iter()
        .any(|local| local.name == identifier && local.depth == self.scope_depth)
      {
        self.error(Error::VariableAlreadyExists, span, identifier);
      } else {
        locals.push(Local {
          name: identifier,
          depth: self.scope_depth,
          closed: false,
        });
        return locals.len().saturating_sub(1);
      }
    } else {
      self.emit_opcode(span, OpCode::DefineGlobal);
      self.emit_constant_string(span, identifier);
    }

    0
  }

  fn import(&mut self, module: &'s str, item: &'s str, span: Span) {
    if let Some(constant) = self.import_cache.get(&(module, item)) {
      self.emit_constant(span, constant.clone());
      return;
    }

    match self.context.get_value(module, item) {
      ImportValue::Constant(value) => {
        self.emit_constant(span, value.clone());
        self.import_cache.insert((module, item), value);
      }
      ImportValue::Bytecode(create_function) => {
        let line = span.get_line_number(self.source);
        let creator = BytecodeFunctionCreator::new(line);
        let function: Value = create_function(creator).into();

        self.emit_constant(span, function.clone());
        self.import_cache.insert((module, item), function);
      }
      ImportValue::ModuleNotFound => self.error(Error::ModuleNotFound, span, module),
      ImportValue::ItemNotFound => self.error(Error::ItemNotFound, span, item),
    };
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

pub fn compile(source: &str, context: &dyn Context) -> Result<Rc<Chunk>, Diagnostic> {
  let parser = Parser::new(source);
  let mut compiler = Compiler::new(source, context);

  for statement in parser {
    compiler.compile_statement(&statement?);

    if let Some(error) = compiler.error {
      return Err(error);
    }
  }

  Ok(compiler.finish().into())
}
