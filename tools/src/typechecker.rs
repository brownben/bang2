// Bidirectional Typechecker for Bang
//
// Based on https://github.com/JDemler/BidirectionalTypechecking, which implements the
// paper "Complete and Easy Bidirectional Typechecking for Higher-Rank Polymorphism"
//
// It doesn't yet support all features of bang. The current known issues are:
// - Doesn't support corecursion, or accessing globals before they are defined

use crate::builtins::{define_globals, get_builtin_module_type};
use ahash::AHashMap as HashMap;
use bang_syntax::{
  ast::{
    expression::{
      AssignmentOperator, BinaryOperator, Expr, Expression, LiteralType, UnaryOperator,
    },
    statement::{DeclarationIdentifier, Statement, Stmt},
    types::{Type as TypeItem, TypeExpression},
    Span,
  },
  Diagnostic,
};
use std::collections::hash_map::Entry as HashMapEntry;
use std::fmt;

type Existential = u32;

const NULL_TYPE: Type = Type::Literal(LiteralType::Null);
const NUMBER_TYPE: Type = Type::Literal(LiteralType::Number);
const STRING_TYPE: Type = Type::Literal(LiteralType::String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Function {
  parameters: Vec<Type>,
  return_type: Type,
  catch_all: bool,
}

enum Truthiness {
  True,
  False,
  Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
  Literal(LiteralType),
  Boolean,
  Any,
  Never,
  Existential(Existential),
  Function(Box<Function>),
  Union(Box<Self>, Box<Self>),
  List(Box<Self>),
}
impl Type {
  fn includes(&self, alpha: Existential) -> bool {
    match self {
      Self::Any | Self::Boolean | Self::Literal(_) | Self::Never => false,
      Self::Function(function) => {
        function
          .parameters
          .iter()
          .any(|parameter| parameter.includes(alpha))
          || function.return_type.includes(alpha)
      }
      Self::Existential(var) => *var == alpha,
      Self::Union(a, b) => a.includes(alpha) || b.includes(alpha),
      Self::List(element_type) => element_type.includes(alpha),
    }
  }

  fn includes_type(&self, t: &Self) -> bool {
    match (self, t) {
      (a, b) if a == b => true,
      (Self::Any, _) | (Self::Boolean, Self::Literal(LiteralType::True | LiteralType::False)) => {
        true
      }
      (Self::Union(c, d), b) => c.includes_type(b) || d.includes_type(b),
      _ => false,
    }
  }

  fn is_truthy(&self) -> bool {
    match self {
      Self::Literal(LiteralType::True) | Self::Function(_) => true,
      Self::Union(a, b) => a.is_truthy() && b.is_truthy(),
      _ => false,
    }
  }

  fn is_falsy(&self) -> bool {
    match self {
      Self::Literal(LiteralType::False | LiteralType::Null) => true,
      Self::Union(a, b) => a.is_falsy() && b.is_falsy(),
      _ => false,
    }
  }

  fn truthiness(&self) -> Truthiness {
    if self.is_truthy() {
      Truthiness::True
    } else if self.is_falsy() {
      Truthiness::False
    } else {
      Truthiness::Unknown
    }
  }

  fn uplevel_literal_booleans(self) -> Self {
    if let Self::Literal(LiteralType::True | LiteralType::False) = self {
      Self::Boolean
    } else {
      self
    }
  }

  pub fn union(a: Self, b: Self) -> Self {
    match (a, b) {
      (a, b) if a.includes_type(&b) => a,
      (a, Self::Never) => a,
      (Self::Never, b) => b,
      (a, b) => Self::Union(Box::new(a), Box::new(b)),
    }
  }

  fn narrow(self, type_: &Self) -> Self {
    match (self, type_) {
      (a, b) if a == *b => Self::Never,
      (Self::Union(a, b), Self::Union(c, d)) => {
        Self::union(a.narrow(c).narrow(d), b.narrow(c).narrow(d))
      }
      (Self::Union(a, b), type_) => Self::union(a.narrow(type_), b.narrow(type_)),
      (Self::Boolean, Self::Literal(LiteralType::True)) => Self::Literal(LiteralType::False),
      (Self::Boolean, Self::Literal(LiteralType::False)) => Self::Literal(LiteralType::True),
      (x, _) => x,
    }
  }

  pub fn function(parameters: Vec<Self>, return_type: Self, catch_all: bool) -> Self {
    Self::Function(Box::new(Function {
      parameters,
      return_type,
      catch_all,
    }))
  }
}
impl fmt::Display for Type {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match self {
      Self::Any => write!(f, "any"),
      Self::Never => write!(f, "never"),
      Self::Boolean => write!(f, "boolean"),
      Self::Literal(lit) => write!(f, "{lit}"),
      Self::Existential(ex) => write!(f, "^{ex}"),
      Self::Union(a, b) => write!(f, "{a} | {b}"),
      Self::Function(function) => write!(
        f,
        "({}) -> {}",
        function
          .parameters
          .iter()
          .map(std::string::ToString::to_string)
          .collect::<Vec<_>>()
          .join(", "),
        function.return_type
      ),
      Self::List(element_type) => {
        if let Self::Union(_, _) | Self::Function(_) = **element_type {
          write!(f, "({element_type})[]")
        } else {
          write!(f, "{element_type}[]")
        }
      }
    }
  }
}

#[derive(Clone, Debug)]
struct Solved {
  existential: Existential,
  type_: Type,
  scope: u8,
}

#[derive(Clone)]
struct Variable<'s> {
  name: &'s str,
  type_: Type,
  scope: u8,
}
impl fmt::Debug for Variable<'_> {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    write!(f, "({name}: {type_})", name = self.name, type_ = self.type_,)
  }
}

#[derive(Clone, Debug)]
struct Restriction<'s>(&'s str, Type);

enum Error {
  ExpectedType,
  NotCallable,
  WrongNumberArguments,
  UnknownType,
  UnknownVariable,
  BuiltinNotFound,
}
impl Error {
  fn get_title(&self) -> &'static str {
    match self {
      Self::ExpectedType => "Expected Types to Match",
      Self::NotCallable => "Type Not Callable",
      Self::WrongNumberArguments => "Wrong Number of Arguments",
      Self::UnknownType => "Unknown Type",
      Self::UnknownVariable => "Unknown Variable",
      Self::BuiltinNotFound => "Builtin Not Found",
    }
  }

  fn into_diagnostic(self, message: String, span: Span, source: &str) -> Diagnostic {
    Diagnostic {
      title: self.get_title().to_string(),
      message,
      line: span.get_line_number(source),
      span,
    }
  }
}

pub struct Typechecker<'source> {
  source: &'source str,
  errors: Vec<Diagnostic>,

  scope: u8,
  solved_scope: u8,
  existential_id: Existential,

  solved: Vec<Solved>,
  variables: Vec<Variable<'source>>,
  existentials: Vec<Existential>,
}
impl<'s> Typechecker<'s> {
  pub fn new(source: &'s str) -> Self {
    Self {
      source,
      errors: Vec::new(),

      scope: 0,
      solved_scope: 0,
      existential_id: 0,

      solved: Vec::new(),
      variables: Vec::new(),
      existentials: Vec::new(),
    }
  }

  fn error(&mut self, error: Error, message: String, span: Span) -> Type {
    self
      .errors
      .push(error.into_diagnostic(message, span, self.source));

    Type::Never
  }

  fn type_from_annotation(&mut self, annotation: &TypeExpression) -> Type {
    match &annotation.type_ {
      TypeItem::Named(name) => match *name {
        "string" => STRING_TYPE,
        "number" => NUMBER_TYPE,
        "null" => NULL_TYPE,
        "false" => Type::Literal(LiteralType::False),
        "true" => Type::Literal(LiteralType::True),
        "boolean" => Type::Boolean,
        "any" => Type::Any,
        _ => self.error(
          Error::UnknownType,
          format!("Unknown type '{name}'"),
          annotation.span,
        ),
      },
      TypeItem::Group(t) => self.type_from_annotation(t),
      TypeItem::Function(return_type, parameters) => {
        let return_type = self.type_from_annotation(return_type);
        let parameters = parameters
          .iter()
          .map(|p| self.type_from_annotation(p))
          .collect::<Vec<_>>();

        Type::function(parameters, return_type, false)
      }
      TypeItem::Union(a, b) => {
        Type::union(self.type_from_annotation(a), self.type_from_annotation(b))
      }
      TypeItem::Optional(t) => Type::union(NULL_TYPE, self.type_from_annotation(t)),
      TypeItem::List(t) => Type::List(Box::new(self.type_from_annotation(t))),
    }
  }

  fn begin_scope(&mut self) {
    self.scope += 1;
  }

  pub(crate) fn define(&mut self, name: &'s str, type_: &Type) {
    self.variables.push(Variable {
      name,
      type_: self.apply_context(type_),
      scope: self.scope,
    });
  }

  fn get_variable(&self, x: &str) -> Option<&Type> {
    for variable in self.variables.iter().rev() {
      if variable.name == x {
        return Some(&variable.type_);
      }
    }
    None
  }

  fn end_scope(&mut self) {
    loop {
      if self.variables.last().is_some() && self.variables.last().unwrap().scope == self.scope {
        self.variables.pop();
      } else {
        break;
      }
    }

    self.scope -= 1;
  }

  fn begin_solved_scope(&mut self) {
    self.solved_scope += 1;
  }

  fn define_existential(&mut self, existential: Existential, type_: Type) {
    self.solved.push(Solved {
      existential,
      type_,
      scope: self.scope,
    });
  }

  fn get_solved(&self, alpha: Existential) -> Option<&Type> {
    self
      .solved
      .iter()
      .rfind(|solved| solved.existential == alpha)
      .map(|solved| &solved.type_)
  }

  fn end_solved_scope(&mut self) {
    loop {
      if self.solved.last().is_some() && self.solved.last().unwrap().scope == self.scope {
        self.solved.pop();
      } else {
        break;
      }
    }

    self.solved_scope -= 1;
  }

  pub fn new_existential(&mut self) -> Type {
    self.existential_id += 1;
    self.existentials.push(self.existential_id);
    Type::Existential(self.existential_id)
  }

  pub fn apply_context(&self, type_: &Type) -> Type {
    match type_ {
      Type::Never => Type::Never,
      Type::Any => Type::Any,
      Type::Boolean => Type::Boolean,
      Type::Literal(_) => type_.clone(),
      Type::Existential(alpha) => {
        if let Some(tau) = self.get_solved(*alpha) {
          self.apply_context(tau)
        } else {
          type_.clone()
        }
      }
      Type::Function(function) => Type::function(
        function
          .parameters
          .iter()
          .map(|parameter| self.apply_context(parameter))
          .collect(),
        self.apply_context(&function.return_type),
        function.catch_all,
      ),
      Type::Union(a, b) => Type::union(self.apply_context(a), self.apply_context(b)),
      Type::List(type_) => Type::List(Box::new(self.apply_context(type_))),
    }
  }

  fn subtype(&mut self, a: &Type, b: &Type) -> bool {
    match (a, b) {
      (_, Type::Never) => false,
      (_, Type::Any) => true,
      (a, b) if a == b => true,
      (Type::Existential(alpha), b) => {
        if b.includes(*alpha) {
          false
        } else {
          // Create a subtype of b
          self.define_existential(*alpha, b.clone());
          true
        }
      }
      (a, Type::Existential(alpha)) => {
        if a.includes(*alpha) {
          false
        } else {
          // Create a supertype of a
          self.define_existential(*alpha, a.clone());
          true
        }
      }
      (Type::Literal(LiteralType::True | LiteralType::False), Type::Boolean) => true,
      (Type::List(a), Type::List(b)) => self.subtype(a, b),
      (Type::Union(a, b), x) => self.subtype(a, x) && self.subtype(b, x),
      (x, Type::Union(a, b)) => self.subtype(x, a) || self.subtype(x, b),
      (Type::Function(a), Type::Function(b)) => {
        a.parameters.len() == b.parameters.len()
          && a
            .parameters
            .iter()
            .zip(b.parameters.iter())
            .all(|(a, b)| self.subtype(b, a))
          && self.subtype(
            &self.apply_context(&a.return_type),
            &self.apply_context(&b.return_type),
          )
      }
      _ => false,
    }
  }

  fn check_statement(&mut self, stmt: &Statement<'s>, type_: &Type) {
    let stmt_type = match self.synthesize_statement(stmt) {
      Some(ty) => self.apply_context(&ty),
      None => NULL_TYPE,
    };

    self.check_type(&stmt_type, type_, stmt.span);
  }

  fn check_expression(&mut self, expr: &Expression<'s>, type_: &Type) {
    let expression_type = self.synthesize_expression(expr);
    let expression_type = self.apply_context(&expression_type);
    let type_ = self.apply_context(type_);

    self.check_type(&expression_type, &type_, expr.span);
  }

  fn check_type(&mut self, got: &Type, expected: &Type, span: Span) {
    if !self.subtype(got, expected) {
      self.error(
        Error::ExpectedType,
        format!("Expected type {expected}, but recieved {got}"),
        span,
      );
    }
  }

  pub fn synthesize_statement(&mut self, stmt: &Statement<'s>) -> Option<Type> {
    match &stmt.stmt {
      Stmt::Declaration {
        identifier,
        expression,
        type_,
      } => {
        let annotation = if let Some(annotation) = type_ {
          self.type_from_annotation(annotation)
        } else if let Some(expression) = expression {
          self
            .synthesize_expression(expression)
            .uplevel_literal_booleans()
        } else {
          NULL_TYPE
        };

        if let Some(expression) = expression {
          self.check_expression(expression, &annotation);
        } else {
          self.check_type(&NULL_TYPE, &annotation, stmt.span);
        }

        match identifier {
          DeclarationIdentifier::Variable(identifier) => self.define(identifier, &annotation),
          DeclarationIdentifier::List(identifiers) => identifiers.iter().for_each(|identifier| {
            if annotation == STRING_TYPE {
              self.define(identifier, &annotation);
            } else {
              let list_inner_type = self.new_existential();
              self.check_type(
                &annotation,
                &Type::List(Box::new(list_inner_type.clone())),
                stmt.span,
              );
              self.define(identifier, &list_inner_type);
            }
          }),
        }

        None
      }
      Stmt::If {
        condition,
        then,
        otherwise,
      } => {
        let condition_type = self.synthesize_expression(condition);
        let restrictions = self.get_restrictions(condition);
        let x = self.synthesize_statement_with_restriction(then, restrictions);

        if let Some(otherwise) = otherwise {
          let restrictions = self.get_inverse_restrictions(condition);
          let y = self.synthesize_statement_with_restriction(otherwise, restrictions);

          match (condition_type.truthiness(), x, y) {
            (Truthiness::True, x, _) => x,
            (Truthiness::False, _, y) => y,
            (_, Some(x), Some(y)) => Some(Type::union(x, y)),
            (_, Some(x), None) => Some(x),
            (_, None, Some(y)) => Some(y),
            (_, None, None) => None,
          }
        } else if condition_type.is_falsy() {
          None
        } else {
          x
        }
      }
      Stmt::While { condition, body } => {
        let condition_type = self.synthesize_expression(condition);
        let restrictions = self.get_restrictions(condition);

        if condition_type.is_falsy() {
          None
        } else {
          self.synthesize_statement_with_restriction(body, restrictions)
        }
      }
      Stmt::Block { body, .. } => {
        self.begin_scope();
        for statement in body {
          let returns = self.synthesize_statement(statement);

          if returns.is_some() {
            self.end_scope();
            return returns;
          }
        }
        self.end_scope();
        None
      }
      Stmt::Expression { expression, .. } => {
        self.synthesize_expression(expression);
        None
      }
      Stmt::Comment { .. } => None,
      Stmt::Import { module, items } => {
        for item in items {
          if let Some(type_) = get_builtin_module_type(self, module, item.name) {
            self.define(item.alias.unwrap_or(item.name), &type_);
          } else {
            self.error(
              Error::BuiltinNotFound,
              format!("Couldn't find '{}' in module '{module}'", item.name),
              item.span,
            );
          }
        }
        None
      }
      Stmt::Return { expression } => {
        if let Some(expression) = expression {
          Some(self.synthesize_expression(expression))
        } else {
          Some(NULL_TYPE)
        }
      }
    }
  }

  fn synthesize_statement_with_restriction(
    &mut self,
    statement: &Statement<'s>,
    restrictions: Vec<Restriction<'s>>,
  ) -> Option<Type> {
    self.begin_scope();
    self.begin_solved_scope();

    for Restriction(name, type_) in restrictions {
      self.define(name, &type_);
    }
    let returns = self.synthesize_statement(statement);

    self.end_scope();
    self.end_solved_scope();
    returns
  }

  pub fn synthesize_expression(&mut self, expr: &Expression<'s>) -> Type {
    let span = expr.span;

    match &expr.expr {
      Expr::Literal { type_, .. } => Type::Literal(*type_),
      Expr::FormatString { expressions, .. } => {
        for expr in expressions {
          self.synthesize_expression(expr);
        }
        Type::Literal(LiteralType::String)
      }
      Expr::Comment { expression, .. } | Expr::Group { expression } => {
        self.synthesize_expression(expression)
      }
      Expr::Variable { name } => {
        if let Some(type_) = self.get_variable(name) {
          type_.clone()
        } else {
          self.error(
            Error::UnknownVariable,
            format!("Variable '{name}' is undefined"),
            span,
          )
        }
      }
      Expr::Assignment {
        identifier,
        expression,
        ..
      } => {
        if let Some(type_) = self.get_variable(identifier) {
          let t = type_.clone();
          self.check_expression(expression, &t);
          t
        } else {
          self.error(
            Error::UnknownVariable,
            format!("Variable '{identifier}' is undefined"),
            span,
          )
        }
      }

      Expr::Function {
        parameters,
        body,
        return_type,
        name,
      } => {
        let return_type = if let Some(rt) = return_type {
          self.type_from_annotation(rt)
        } else {
          self.new_existential()
        };
        let arg_types = parameters
          .iter()
          .map(|param| {
            if let Some(ty) = &param.type_ {
              self.type_from_annotation(ty)
            } else {
              self.new_existential()
            }
          })
          .collect::<Vec<_>>();
        let has_catch_all_parameter = parameters
          .last()
          .map_or(false, |parameter| parameter.catch_remaining);

        self.begin_scope();
        for (type_, param) in arg_types.iter().zip(parameters.iter()) {
          self.define(param.name, type_);
        }
        if let Some(name) = name {
          let function = Type::function(
            arg_types.clone(),
            self.apply_context(&return_type),
            has_catch_all_parameter,
          );
          self.define(name, &function);
        };
        self.check_statement(body, &return_type);
        self.end_scope();

        Type::function(
          arg_types
            .iter()
            .map(|type_| self.apply_context(type_))
            .collect(),
          self.apply_context(&return_type),
          has_catch_all_parameter,
        )
      }

      Expr::Call {
        expression,
        arguments,
      } => {
        let expression = self.synthesize_expression(expression);
        let return_type = self.synthesize_application(&expression, arguments, span);

        self.apply_context(&return_type)
      }

      Expr::Unary {
        operator,
        expression,
        ..
      } => {
        let type_ = self.synthesize_expression(expression);
        match operator {
          UnaryOperator::Minus => {
            self.check_expression(expression, &NUMBER_TYPE);
            NUMBER_TYPE
          }
          UnaryOperator::Not => match type_ {
            type_ if type_.is_truthy() => Type::Literal(LiteralType::False),
            type_ if type_.is_falsy() => Type::Literal(LiteralType::True),
            _ => Type::Boolean,
          },
        }
      }

      Expr::Binary {
        operator,
        left,
        right,
      } => {
        if let BinaryOperator::Pipeline = operator {
          return self.synthesize_pipeline(left, right);
        }

        let l = self.synthesize_expression(left);
        let r = self.synthesize_expression(right);

        match operator {
          BinaryOperator::Plus => {
            self.check_expression(
              left,
              &Type::Union(Box::new(NUMBER_TYPE), Box::new(STRING_TYPE)),
            );
            self.check_expression(right, &l);
            l
          }
          BinaryOperator::Minus | BinaryOperator::Multiply | BinaryOperator::Divide => {
            self.check_expression(left, &NUMBER_TYPE);
            self.check_expression(right, &NUMBER_TYPE);
            NUMBER_TYPE
          }
          BinaryOperator::Equal | BinaryOperator::NotEqual => {
            self.check_expression(right, &l);
            Type::Boolean
          }
          BinaryOperator::Greater
          | BinaryOperator::Less
          | BinaryOperator::GreaterEqual
          | BinaryOperator::LessEqual => {
            self.check_expression(
              left,
              &Type::Union(Box::new(NUMBER_TYPE), Box::new(STRING_TYPE)),
            );
            self.check_expression(right, &l);
            Type::Boolean
          }
          BinaryOperator::And => match &l {
            type_ if type_.is_falsy() => l,
            type_ if type_.is_truthy() => r,
            _ => Type::union(l, r),
          },
          BinaryOperator::Or => match &l {
            type_ if type_.is_falsy() => r,
            type_ if type_.is_truthy() => l,
            _ => Type::union(l, r),
          },
          BinaryOperator::Nullish => {
            if self.subtype(&NULL_TYPE, &l) {
              Type::union(l.narrow(&NULL_TYPE), r)
            } else {
              l
            }
          }
          BinaryOperator::Pipeline => unreachable!(),
        }
      }

      Expr::List { items } => {
        if items.is_empty() {
          return Type::List(Box::new(self.new_existential()));
        }

        Type::List(Box::new(
          items
            .iter()
            .map(|item| self.synthesize_expression(item))
            .fold(Type::Never, Type::union),
        ))
      }
      Expr::Index { expression, index } => {
        self.check_expression(index, &NUMBER_TYPE);

        let expression_type = self.synthesize_expression(expression);
        if expression_type == STRING_TYPE {
          return STRING_TYPE;
        }

        let list_interior = self.new_existential();
        self.check_expression(expression, &Type::List(Box::new(list_interior.clone())));
        list_interior
      }
      Expr::IndexAssignment {
        expression,
        index,
        value,
        assignment_operator,
      } => {
        let list_interior = self.new_existential();
        self.check_expression(expression, &Type::List(Box::new(list_interior.clone())));
        self.check_expression(index, &NUMBER_TYPE);
        self.check_expression(value, &list_interior);

        match assignment_operator {
          Some(AssignmentOperator::Plus) => {
            let plus_able = Type::union(NUMBER_TYPE, STRING_TYPE);
            self.check_expression(value, &plus_able);
            self.check_expression(expression, &Type::List(Box::new(plus_able)));
          }
          Some(_) => {
            self.check_expression(value, &NUMBER_TYPE);
            self.check_expression(expression, &Type::List(Box::new(NUMBER_TYPE)));
          }
          _ => (),
        };

        list_interior
      }
    }
  }

  fn synthesize_pipeline(&mut self, left: &Expression<'s>, right: &Expression<'s>) -> Type {
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

      (self.synthesize_expression(expression), arguments)
    } else {
      (self.synthesize_expression(right), vec![left.clone()])
    };

    let return_type = self.synthesize_application(&expression, &arguments, left.span);
    self.apply_context(&return_type)
  }

  fn synthesize_application(
    &mut self,
    expression: &Type,
    arguments: &[Expression<'s>],
    span: Span,
  ) -> Type {
    self.begin_solved_scope();

    let type_ = match expression {
      Type::Existential(alpha) => {
        let alpha_args: Vec<_> = (0..arguments.len())
          .map(|_| self.new_existential())
          .collect();
        let return_type = self.new_existential();

        self.define_existential(
          *alpha,
          Type::function(alpha_args.clone(), return_type.clone(), false),
        );

        for (alpha, expression) in alpha_args.iter().zip(arguments.iter()) {
          self.check_expression(expression, alpha);
        }

        return_type
      }
      Type::Function(function) => {
        if (function.catch_all && arguments.len() < function.parameters.len() - 1)
          || (!function.catch_all && arguments.len() != function.parameters.len())
        {
          return self.error(
            Error::WrongNumberArguments,
            format!(
              "Expected {} arguments, got {}",
              function.parameters.len(),
              arguments.len()
            ),
            span,
          );
        }

        let normal_parameter_end_index =
          function.parameters.len() - if function.catch_all { 1 } else { 0 };

        function.parameters[..normal_parameter_end_index]
          .iter()
          .zip(arguments.iter())
          .for_each(|(arg, e)| self.check_expression(e, arg));

        if function.catch_all {
          let items = arguments[(function.parameters.len() - 1)..].to_owned();

          self.check_expression(
            &Expression {
              expr: Expr::List { items },
              span,
            },
            function.parameters.last().unwrap(),
          );
        }

        function.return_type.clone()
      }
      _ => self.error(
        Error::NotCallable,
        format!("Type '{expression}' is not callable"),
        span,
      ),
    };

    let type_ = self.apply_context(&type_);
    self.end_solved_scope();
    type_
  }
}

/// Restrictions
impl<'s> Typechecker<'s> {
  fn get_restrictions(&mut self, expression: &Expression<'s>) -> Vec<Restriction<'s>> {
    match &expression.expr {
      Expr::Group { expression } | Expr::Comment { expression, .. } => {
        return self.get_restrictions(expression);
      }
      Expr::Binary {
        operator: BinaryOperator::Equal,
        left,
        right,
      } => {
        let r = self.synthesize_expression(right);

        if let Expr::Variable { name } = &left.expr {
          return vec![Restriction(name, r)];
        }
      }
      Expr::Binary {
        operator: BinaryOperator::NotEqual,
        left,
        right,
      } => {
        let l = self.synthesize_expression(left);
        let r = self.synthesize_expression(right);

        if let Expr::Variable { name } = &left.expr &&
          let Type::Literal(LiteralType::Null | LiteralType::True | LiteralType::False) = &r
        {
          return vec![Restriction(name, l.narrow(&r))];
        }
      }
      Expr::Binary {
        operator: BinaryOperator::And,
        left,
        right,
      } => {
        let l = self.get_restrictions(left);
        let r = self.get_restrictions(right);

        let mut restrictions = l
          .into_iter()
          .map(|Restriction(name, type_)| (name, type_))
          .collect::<HashMap<_, _>>();

        for Restriction(name, restriction) in r {
          if let HashMapEntry::Occupied(mut entry) = restrictions.entry(name) {
            entry.insert(restriction.narrow(entry.get()));
          } else {
            restrictions.insert(name, restriction);
          }
        }

        return restrictions
          .into_iter()
          .map(|(name, restriction)| Restriction(name, restriction))
          .collect::<Vec<_>>();
      }
      Expr::Binary {
        operator: BinaryOperator::Or,
        left,
        right,
      } => {
        let mut restrictions = self.get_restrictions(left);
        restrictions.append(&mut self.get_restrictions(right));

        // If different variables are on each side of the OR we can't make a decision
        let first_name = restrictions[0].0;
        let all_same_name = restrictions
          .iter()
          .all(|Restriction(name, _)| *name == first_name);

        if all_same_name {
          return vec![Restriction(
            first_name,
            restrictions
              .iter()
              .map(|Restriction(_, t)| t)
              .fold(Type::Never, |accum, x| Type::union(accum, x.clone())),
          )];
        }
      }
      _ => {}
    }

    Vec::new()
  }

  fn get_inverse_restrictions(&mut self, expression: &Expression<'s>) -> Vec<Restriction<'s>> {
    let restrictions = self.get_restrictions(expression);

    restrictions
      .into_iter()
      .map(|Restriction(name, not_type)| {
        let current_type = self
          .get_variable(name)
          .expect("variable to exist as previously found")
          .clone();

        Restriction(name, current_type.narrow(&not_type))
      })
      .collect()
  }
}

pub fn typecheck<'s>(source: &'s str, ast: &[Statement]) -> Vec<Diagnostic> {
  let mut typechecker = Typechecker::new(source);
  define_globals(&mut typechecker);

  for statement in ast {
    typechecker.synthesize_statement(statement);
  }

  typechecker.errors.dedup();
  typechecker.errors
}
