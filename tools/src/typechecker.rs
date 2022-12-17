#[macro_use]
mod builtins;
mod expressions;
mod narrowing;
mod statements;
mod types;

use statements::StatementType;
use types::{Existential, Function, Literal, Type};

use bang_syntax::ast::{
  expression::{Expr, Expression},
  statement::{Statement, Stmt},
  types::{Type as TypeItem, TypeExpression},
  Span,
};
use rustc_hash::FxHashMap as HashMap;
use std::{error, fmt, mem};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorKind {
  UnknownType(String),
  UndefinedVariable(String),
  VariableAlreadyDefined(String),
  ExpectedDifferentType(Type, Type),
  ImportItemNotFound(String),
  ImportModuleNotFound(String),
  NotCallable(Type),
  WrongNumberArguments(usize, usize),
  WrongNumberTypeParameters(usize, usize),
  InfiniteLoop,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
  kind: ErrorKind,
  pub span: Span,
}
impl Error {
  fn new(kind: ErrorKind, span: Span) -> Result<Type, Self> {
    Err(Self { kind, span })
  }

  pub fn get_title(&self) -> &'static str {
    match self.kind {
      ErrorKind::UnknownType(_) => "Unknown Type",
      ErrorKind::UndefinedVariable(_) => "Undefined Variable",
      ErrorKind::VariableAlreadyDefined(_) => "Variable Already Defined",
      ErrorKind::ExpectedDifferentType(_, _) => "Expected Different Type",
      ErrorKind::ImportItemNotFound(_) => "Item Not Found In Module",
      ErrorKind::ImportModuleNotFound(_) => "Module Not Found",
      ErrorKind::NotCallable(_) => "Type Not Callable",
      ErrorKind::WrongNumberArguments(_, _) => "Incorrect Number of Arguments",
      ErrorKind::WrongNumberTypeParameters(_, _) => "Incorrect Number of Type Parameters",
      ErrorKind::InfiniteLoop => "Infinite Loop",
    }
  }

  pub fn get_description(&self) -> String {
    match &self.kind {
      ErrorKind::UnknownType(ty) => format!("Unknown type '{ty}'."),
      ErrorKind::UndefinedVariable(var) => format!("Undefined variable '{var}'."),
      ErrorKind::VariableAlreadyDefined(var) => {
        format!("Variable '{var}' has already been defined.")
      }
      ErrorKind::ExpectedDifferentType(a, b) => {
        format!("Expected type '{b}' but recieved '{a}'.")
      }
      ErrorKind::ImportItemNotFound(item) => format!("Item '{item}' not found in module."),
      ErrorKind::ImportModuleNotFound(module) => format!("Module '{module}' not found."),
      ErrorKind::NotCallable(ty) => {
        format!("Type {ty} is not callable. Only functions are callable.")
      }
      ErrorKind::WrongNumberArguments(a, b) => format!("Expected {b} arguments, but recieved {a}."),
      ErrorKind::WrongNumberTypeParameters(a, b) => {
        format!("Expected {b} type parameters, but recieved {a}.")
      }
      ErrorKind::InfiniteLoop => {
        "Condition is always true and there is no return in the loop.".to_string()
      }
    }
  }
}
impl error::Error for Error {}
impl fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.get_title())
  }
}

type ScopeDepth = u16;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Variable<'s> {
  name: &'s str,
  initalization: bool,
  depth: ScopeDepth,
  ty: Type,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Scope<'s> {
  variables: Vec<Variable<'s>>,
  depth: ScopeDepth,
}
impl<'s> Scope<'s> {
  fn define(&mut self, name: &'s str, ty: Type, span: Span) -> Result<(), Error> {
    if self.is_defined(name) {
      Error::new(ErrorKind::VariableAlreadyDefined(name.to_string()), span)?;
    }

    self.insert(name, ty);

    Ok(())
  }
  fn insert(&mut self, name: &'s str, ty: Type) {
    self.variables.push(Variable {
      name,
      ty,
      initalization: true,
      depth: self.depth,
    });
  }
  fn update(&mut self, name: &'s str, ty: Type) {
    self.variables.push(Variable {
      name,
      ty,
      initalization: false,
      depth: self.depth,
    });
  }

  fn lookup(&self, name: &'s str) -> Option<Type> {
    self
      .variables
      .iter()
      .rfind(|variable| variable.name == name)
      .map(|variable| variable.ty.clone())
  }
  fn is_defined(&self, name: &'s str) -> bool {
    self
      .variables
      .iter()
      .rfind(|variable| {
        variable.name == name && variable.depth == self.depth && variable.initalization
      })
      .is_some()
  }

  fn lookup_initialization(&self, name: &'s str) -> Option<Type> {
    self
      .variables
      .iter()
      .rfind(|variable| variable.name == name && variable.initalization)
      .map(|variable| variable.ty.clone())
  }

  fn begin_scope(&mut self) {
    self.depth += 1;
  }
  fn end_scope(&mut self) {
    while let Some(last) = self.variables.last() && last.depth >= self.depth {
      self.variables.pop();
    }
    self.depth -= 1;
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ContextItem {
  existential: Existential,
  ty: Type,
  depth: ScopeDepth,
}
#[derive(Default, Debug, Clone)]
pub struct Context {
  existential_count: Existential,
  context: Vec<ContextItem>,
  depth: ScopeDepth,
}
impl Context {
  fn new_existential(&mut self) -> Type {
    self.existential_count += 1;
    Type::Existential(self.existential_count)
  }

  fn solve(&mut self, existential: Existential, ty: Type) {
    self.context.push(ContextItem {
      existential,
      ty,
      depth: self.depth,
    });
  }
  fn lookup(&self, existential: Existential) -> Option<Type> {
    self
      .context
      .iter()
      .rfind(|variable| variable.existential == existential)
      .map(|variable| variable.ty.clone())
  }

  fn begin_scope(&mut self) {
    self.depth += 1;
  }
  fn end_scope(&mut self) {
    while let Some(ContextItem { depth, .. }) = self.context.last() && *depth == self.depth {
      self.context.pop();
    }
    self.depth -= 1;
  }
}

#[derive(Default, Debug, Clone)]
struct Typechecker<'s> {
  scope: Scope<'s>,
  context: Context,
}
impl<'s> Typechecker<'s> {
  fn type_from_annotation(
    &mut self,
    annotation: &TypeExpression<'s>,
    generics: &mut HashMap<&'s str, Type>,
  ) -> Result<Type, Error> {
    let span = annotation.span;
    let ty = match &annotation.type_ {
      TypeItem::Named(name) => match *name {
        "string" => Type::Literal(Literal::String),
        "number" => Type::Literal(Literal::Number),
        "false" => Type::Literal(Literal::False),
        "true" => Type::Literal(Literal::True),
        "null" => Type::NULL,
        "boolean" => Type::boolean(),
        "any" => Type::Any,
        _ if generics.contains_key(name) => generics[name].clone(),
        ty => Error::new(ErrorKind::UnknownType(ty.to_string()), annotation.span)?,
      },
      TypeItem::Parameter(name, param) => {
        let expected_params = match *name {
          "set" | "list" => 1,
          "dict" => 2,
          ty => return Error::new(ErrorKind::UnknownType(ty.to_string()), annotation.span),
        };

        if param.len() != expected_params {
          Error::new(
            ErrorKind::WrongNumberTypeParameters(param.len(), expected_params),
            span,
          )?;
        }
        match *name {
          "set" => Type::Set(self.type_from_annotation(&param[0], generics)?.into()),
          "list" => Type::List(self.type_from_annotation(&param[0], generics)?.into()),
          "dict" => Type::Dict(
            self.type_from_annotation(&param[0], generics)?.into(),
            self.type_from_annotation(&param[1], generics)?.into(),
          ),
          _ => unreachable!(),
        }
      }
      TypeItem::Union(a, b) => {
        let a = self.type_from_annotation(a, generics)?;
        let b = self.type_from_annotation(b, generics)?;
        a.union(b)
      }
      TypeItem::Function(return_type, parameters, catch_all) => {
        let return_type = self.type_from_annotation(return_type, generics)?.into();
        let mut parameters: Vec<_> = parameters
          .iter()
          .map(|p| self.type_from_annotation(p, generics))
          .collect::<Result<_, _>>()?;

        if *catch_all && let Some(param) = parameters.last_mut() {
          *param = Type::List(mem::take(param).into());
        }

        Type::Function(Function {
          parameters,
          return_type,
          catch_all: *catch_all,
        })
      }
      TypeItem::Optional(ty) => {
        let ty = self.type_from_annotation(ty, generics)?;
        Type::NULL.union(ty)
      }
      TypeItem::Group(ty) => self.type_from_annotation(ty, generics)?,
      TypeItem::List(ty) => Type::List(self.type_from_annotation(ty, generics)?.into()),
      TypeItem::WithGeneric(g, annotation) => {
        generics.extend(g.iter().map(|g| (*g, self.context.new_existential())));
        self.type_from_annotation(annotation, generics)?
      }
    };

    Ok(ty)
  }

  fn subtype(&mut self, a: &Type, b: &Type) -> bool {
    if a.is_subtype_of(b) {
      return true;
    }

    let a = a.clone().apply_context(&self.context);
    let b = b.clone().apply_context(&self.context);

    if a.is_subtype_of(&b) {
      return true;
    }

    match (a, b) {
      (Type::Existential(a), b) => {
        self.context.solve(a, b);
        true
      }
      (a, Type::Existential(b)) => {
        self.context.solve(b, a);
        true
      }

      (Type::List(a), Type::List(b)) | (Type::Set(a), Type::Set(b)) => self.subtype(&a, &b),
      (Type::Dict(a, b), Type::Dict(c, d)) => self.subtype(&a, &c) && self.subtype(&b, &d),

      (Type::Union(a), b) => a.into_iter().all(|a| self.subtype(&a, &b)),
      (a, Type::Union(b)) => b.into_iter().any(|b| self.subtype(&a, &b)),

      (Type::Function(a), Type::Function(b)) => {
        a.parameters.len() == b.parameters.len()
          && a
            .parameters
            .iter()
            .zip(b.parameters.iter())
            .all(|(a, b)| self.subtype(b, a))
          && self.subtype(&a.return_type, &b.return_type)
      }
      (_, _) => false,
    }
  }

  fn assert_type(&mut self, a: Type, b: &Type, span: Span) -> Result<Type, Error> {
    if self.subtype(&a, b) {
      Ok(a)
    } else {
      Error::new(
        ErrorKind::ExpectedDifferentType(
          a.apply_context(&self.context),
          b.clone().apply_context(&self.context),
        ),
        span,
      )
    }
  }

  fn synthesize_expression(&mut self, expression: &Expression<'s>) -> Result<Type, Error> {
    let span = expression.span;

    match &expression.expr {
      Expr::Assignment {
        identifier,
        expression,
      } => self.assignment_expression(expression, identifier, span),
      Expr::Binary {
        operator,
        left,
        right,
      } => self.binary_expression(*operator, left, right, span),
      Expr::Call {
        expression,
        arguments,
      } => {
        let expression = self.synthesize_expression(expression)?;
        self.synthesize_application(&expression, arguments, span)
      }
      Expr::Comment { expression, .. } | Expr::Group { expression } => {
        self.synthesize_expression(expression)
      }
      Expr::Dictionary { items } => self.dictionary_expression(items),
      Expr::FormatString { expressions, .. } => {
        expressions
          .iter()
          .map(|expression| self.synthesize_expression(expression))
          .collect::<Result<Vec<_>, _>>()?;

        Ok(Type::Literal(Literal::String))
      }
      Expr::Function {
        parameters,
        return_type,
        body,
        name,
      } => self.function_expression(*name, parameters, body, return_type, span),
      Expr::Index { index, expression } => self.index_expression(index, expression, span),
      Expr::IndexAssignment {
        expression,
        index,
        value,
        assignment_operator,
      } => self.index_assgnment_expression(expression, index, value, *assignment_operator, span),
      Expr::List { items } => self.list_expression(items),
      Expr::Literal { type_, .. } => Ok(Type::Literal(*type_)),
      Expr::ModuleAccess { module, item } => self.module_access(module, item, span),
      Expr::Unary {
        expression,
        operator,
      } => self.unary_expression(expression, *operator, span),
      Expr::Variable { name, .. } => self.scope.lookup(name).ok_or_else(|| {
        Error::new(ErrorKind::UndefinedVariable((*name).to_string()), span).unwrap_err()
      }),
    }
  }

  fn synthesize_statement(&mut self, statement: &Statement<'s>) -> Result<StatementType, Error> {
    let span = statement.span;

    match &statement.stmt {
      Stmt::Block { body } => self.block_statement(body),
      Stmt::Declaration {
        identifier,
        type_,
        expression,
      } => self.declaration_statement(type_, expression, identifier, span),
      Stmt::Expression { expression } => {
        self.synthesize_expression(expression)?;

        Ok(StatementType::NoReturn)
      }
      Stmt::If {
        condition,
        then,
        otherwise,
      } => self.if_statement(condition, then, otherwise),
      Stmt::Import { module, items } => self.import_statement(items, module),
      Stmt::Return { expression } => self.return_statement(expression),
      Stmt::While { condition, body } => self.while_statement(condition, body),
      Stmt::Comment { .. } => Ok(StatementType::NoReturn),
    }
  }

  fn synthesize_pipeline(
    &mut self,
    left: &Expression<'s>,
    right: &Expression<'s>,
    span: Span,
  ) -> Result<Type, Error> {
    let right = if let Expr::Comment { expression, .. } = &right.expr {
      // If right is a comment, unwrap it
      expression
    } else {
      right
    };

    let (expression, arguments) = if let Expr::Call {
      expression,
      arguments,
      ..
    } = &right.expr
    {
      let mut arguments = arguments.clone();
      arguments.insert(0, left.clone());

      (self.synthesize_expression(expression)?, arguments)
    } else {
      (self.synthesize_expression(right)?, vec![left.clone()])
    };

    self.synthesize_application(&expression, &arguments, span)
  }

  fn synthesize_application(
    &mut self,
    expression: &Type,
    arguments: &[Expression<'s>],
    span: Span,
  ) -> Result<Type, Error> {
    self.context.begin_scope();
    self.scope.begin_scope();

    let type_ = match expression {
      Type::Existential(alpha) => {
        let alpha_args: Vec<_> = (0..arguments.len())
          .map(|_| self.context.new_existential())
          .collect();
        let return_type = self.context.new_existential();

        self.context.solve(
          *alpha,
          Type::Function(Function {
            parameters: alpha_args.clone(),
            return_type: return_type.clone().into(),
            catch_all: false,
          }),
        );

        for (alpha, expression) in alpha_args.iter().zip(arguments.iter()) {
          let ty = self.synthesize_expression(expression)?;
          self.assert_type(ty, alpha, span)?;
        }

        return_type.apply_context(&self.context)
      }
      Type::Function(function) => {
        if (function.catch_all && arguments.len() < function.parameters.len() - 1)
          || (!function.catch_all && arguments.len() != function.parameters.len())
        {
          Error::new(
            ErrorKind::WrongNumberArguments(arguments.len(), function.parameters.len()),
            span,
          )?;
        }

        let normal_parameter_end_index =
          function.parameters.len() - usize::from(function.catch_all);

        function.parameters[..normal_parameter_end_index]
          .iter()
          .zip(arguments.iter())
          .try_for_each(|(arg, e)| {
            let e = self.synthesize_expression(e)?;
            self.assert_type(e, arg, span)?;
            Ok(())
          })?;

        if function.catch_all {
          let items = arguments[(function.parameters.len() - 1)..].to_owned();

          let ty = self.synthesize_expression(&Expression {
            expr: Expr::List { items },
            span,
          })?;
          self.assert_type(ty, function.parameters.last().unwrap(), span)?;
        }

        function.return_type.clone().apply_context(&self.context)
      }
      ty => Error::new(
        ErrorKind::NotCallable(ty.clone().apply_context(&self.context)),
        span,
      )?,
    };

    self.scope.end_scope();
    self.context.end_scope();

    Ok(type_)
  }
}

pub fn typecheck(ast: &[Statement]) -> Vec<Error> {
  let mut typechecker = Typechecker::default();

  register_globals!(&mut typechecker, {
    print: "<T>(T) -> T",
    type: "(any) -> string",
    toString: "(any) -> string",
  });

  ast
    .iter()
    .map(|stmt| typechecker.synthesize_statement(stmt))
    .filter_map(Result::err)
    .collect::<Vec<_>>()
}
