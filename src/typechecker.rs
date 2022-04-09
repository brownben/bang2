// Known Problems in Typechecker:
// - Doesn't support imports
// - Doesn't support corecursion, or accessing globals before they are defined

use crate::{
  ast::{
    expression::{BinaryOperator, Expr, Expression, LiteralType, UnaryOperator},
    statement::{Statement, Stmt},
    types::{Type as TypeItem, TypeExpression},
    Span,
  },
  Diagnostic,
};
use ahash::AHashMap as HashMap;
use std::collections::hash_map::Entry as HashMapEntry;

enum Error {
  ExpectedType,
  NotCallable,
  WrongNumberArguments,
  UnknownType,
  UnknownVariable,
}
impl Error {
  fn get_title(&self) -> &'static str {
    match self {
      Self::ExpectedType => "Expected Types to Match",
      Self::NotCallable => "Type Not Callable",
      Self::WrongNumberArguments => "Wrong Number of Arguments",
      Self::UnknownType => "Unknown Type",
      Self::UnknownVariable => "Unknown Variable",
    }
  }

  fn as_diagnostic(&self, message: String, span: Span, source: &str) -> Diagnostic {
    Diagnostic {
      title: self.get_title().to_string(),
      message,
      lines: vec![span.get_line_number(source)],
    }
  }
}

type TypeIndex = usize;
type Restriction<'s> = (&'s str, TypeIndex);

const NULL: TypeIndex = 0;
const NUMBER: TypeIndex = 1;
const STRING: TypeIndex = 2;
const TRUE: TypeIndex = 3;
const FALSE: TypeIndex = 4;
const BOOLEAN: TypeIndex = 5;
const ANY: TypeIndex = 6;
const NEVER: TypeIndex = 7;
const NUMBER_OR_STRING: TypeIndex = 8;

#[derive(Debug, Clone, Copy)]
struct Variable<'s> {
  name: &'s str,
  depth: u8,
  type_: TypeIndex,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]

enum Type {
  String,
  Number,
  True,
  False,
  Null,

  Any,
  Never,
  Union(Vec<TypeIndex>),
  Function(TypeIndex, Vec<TypeIndex>),
}
impl Type {
  fn get_name(&self, types: &[Type]) -> String {
    match self {
      Self::String => "string".to_string(),
      Self::Number => "number".to_string(),
      Self::True => "true".to_string(),
      Self::False => "false".to_string(),
      Self::Null => "null".to_string(),
      Self::Any => "any".to_string(),
      Self::Never => "never".to_string(),
      Self::Union(parts) => {
        let mut names = parts
          .iter()
          .map(|t| types[*t].get_name(types))
          .collect::<Vec<_>>();
        names.sort();
        names.join(" | ")
      }
      Self::Function(return_type, args) => {
        let mut args = args
          .iter()
          .map(|t| types[*t].get_name(types))
          .collect::<Vec<_>>();
        args.sort();
        format!(
          "({}) -> {}",
          args.join(", "),
          types[*return_type].get_name(types),
        )
      }
    }
  }
}

struct Typechecker<'s> {
  source: &'s str,

  function_stack: Vec<TypeIndex>,
  variables: Vec<Variable<'s>>,
  scope_depth: u8,

  types: Vec<Type>,
  errors: Vec<Diagnostic>,
}
impl<'s> Typechecker<'s> {
  fn new(source: &'s str) -> Self {
    let mut checker = Self {
      source,

      variables: Vec::new(),
      function_stack: Vec::new(),
      scope_depth: 0,

      errors: Vec::new(),
      types: vec![
        Type::Null,
        Type::Number,
        Type::String,
        Type::True,
        Type::False,
        Type::Union(vec![TRUE, FALSE]),
        Type::Any,
        Type::Never,
        Type::Union(vec![STRING, NUMBER]),
      ],
    };

    checker.define_globals();
    checker
  }

  fn define_globals(&mut self) {
    let print = self.add_type(Type::Function(NULL, vec![ANY]));
    let type_ = self.add_type(Type::Function(STRING, vec![ANY]));

    self.define("print", print);
    self.define("type", type_);
  }

  fn add_type(&mut self, type_: Type) -> usize {
    let index = self.types.len();
    self.types.push(type_);
    index
  }

  fn begin_scope(&mut self) {
    self.scope_depth += 1;
  }

  fn end_scope(&mut self) {
    while let Some(Variable {depth, ..}) = self.variables.last()
      && depth == &self.scope_depth
    {
      self.variables.pop();
    }

    self.scope_depth -= 1;
  }

  fn define(&mut self, name: &'s str, type_: TypeIndex) {
    self.variables.push(Variable {
      name,
      depth: self.scope_depth,
      type_,
    });
  }

  fn lookup(&self, name: &'s str) -> Option<&Variable> {
    self.variables.iter().rfind(|local| local.name == name)
  }

  fn error(&mut self, error: Error, message: String, span: Span) -> TypeIndex {
    self
      .errors
      .push(error.as_diagnostic(message, span, self.source));
    NEVER
  }

  fn uplevel(&self, t: TypeIndex) -> TypeIndex {
    match t {
      FALSE => BOOLEAN,
      TRUE => BOOLEAN,
      _ => t,
    }
  }

  fn flatten(&self, t: TypeIndex) -> Vec<TypeIndex> {
    match &self.types[t] {
      Type::Union(parts) => self.flatten_union_members(parts),
      _ => return vec![t],
    }
  }

  fn flatten_union_members(&self, parts: &[TypeIndex]) -> Vec<TypeIndex> {
    let mut type_: Vec<Vec<TypeIndex>> = vec![];
    for t in parts {
      type_.push(self.flatten(*t));
    }
    let mut type_ = type_.concat();
    type_.dedup();
    type_
  }

  fn matches(&self, a_id: TypeIndex, b_id: TypeIndex) -> bool {
    let a = &self.types[a_id];
    let b = &self.types[b_id];

    if *b == Type::Any {
      return true;
    }
    if *b == Type::Never {
      return false;
    }

    match (a, b) {
      (Type::Union(a), Type::Union(_)) => a.iter().all(|item| self.matches(*item, b_id)),
      (Type::Union(a), _) => a.len() == 1 && self.matches(a[0], b_id),
      (_, Type::Union(b)) => b.iter().any(|part| self.matches(a_id, *part)),
      (Type::Function(a, a_args), Type::Function(b, b_args)) => {
        self.matches(*a, *b)
          && a_args.len() == b_args.len()
          && a_args
            .iter()
            .zip(b_args.iter())
            .all(|(a, b)| self.matches(*b, *a))
      }
      (a, b) => a == b,
    }
  }

  fn is_truthy(&self, t: TypeIndex) -> bool {
    let types = self.flatten(t);

    types
      .iter()
      .all(|t| matches!(self.types[*t], Type::True | Type::Function(_, _)))
  }

  fn is_falsy(&self, t: TypeIndex) -> bool {
    let types = self.flatten(t);

    types
      .iter()
      .all(|t| matches!(self.types[*t], Type::False | Type::Null))
  }

  fn narrow(&mut self, a: TypeIndex, b: TypeIndex) -> TypeIndex {
    let types = self
      .flatten(a)
      .into_iter()
      .filter(|t| !self.matches(*t, b))
      .collect::<Vec<_>>();

    match types.len() {
      0 => NEVER,
      1 => types[0],
      _ => self.add_type(Type::Union(types)),
    }
  }

  fn union(&mut self, types: &[TypeIndex]) -> TypeIndex {
    let types = types
      .iter()
      .filter(|t| **t != NEVER)
      .cloned()
      .collect::<Vec<_>>();

    match types.len() {
      0 => NEVER,
      1 => types[0],
      _ => self.add_type(Type::Union(self.flatten_union_members(&types))),
    }
  }

  fn assert_type(&mut self, got: TypeIndex, expected: TypeIndex, span: Span) {
    if !self.matches(got, expected) {
      let got_type = &self.types[got].get_name(&self.types);
      let expected_type = &self.types[expected].get_name(&self.types);

      self.error(
        Error::ExpectedType,
        format!("Expected type '{expected_type}' but received '{got_type}'"),
        span,
      );
    }
  }

  fn type_from_annotation(&mut self, t: &TypeExpression) -> TypeIndex {
    match &t.type_ {
      TypeItem::Named(name) => match *name {
        "string" => STRING,
        "number" => NUMBER,
        "boolean" => BOOLEAN,
        "null" => NULL,
        "false" => FALSE,
        "true" => TRUE,
        "any" => ANY,
        _ => self.error(Error::UnknownType, format!("Unknown type {name}"), t.span),
      },
      TypeItem::Union(a, b) => {
        let a = self.type_from_annotation(a);
        let b = self.type_from_annotation(b);

        self.union(&[a, b])
      }
      TypeItem::Function(return_type, parameters) => {
        let return_type = self.type_from_annotation(return_type);
        let parameters = parameters
          .iter()
          .map(|p| self.type_from_annotation(p))
          .collect::<Vec<_>>();

        self.add_type(Type::Function(return_type, parameters))
      }
      TypeItem::Optional(t) => {
        let t = self.type_from_annotation(t);
        self.union(&[t, NULL])
      }
      TypeItem::Group(t) => self.type_from_annotation(t),
    }
  }

  fn resolve_statement_with_restrictions(
    &mut self,
    statement: &Statement<'s>,
    restrictions: &[Restriction<'s>],
  ) {
    self.begin_scope();
    for (variable, restriction) in restrictions {
      self.define(variable, *restriction);
    }
    self.resolve_statement(statement);
    self.end_scope();
  }

  fn resolve_statement(&mut self, statement: &Statement<'s>) {
    let span = statement.span;

    match &statement.stmt {
      Stmt::Declaration {
        identifier,
        expression,
        type_,
      } => {
        let expression_type = if let Some(expression) = expression {
          let expression_type = self.resolve_expression(expression);
          self.uplevel(expression_type)
        } else {
          NULL
        };

        let annotated_type = if let Some(annotation) = type_ {
          self.type_from_annotation(annotation)
        } else {
          expression_type
        };

        self.assert_type(expression_type, annotated_type, span);
        self.define(identifier, annotated_type)
      }
      Stmt::If {
        then,
        otherwise,
        condition,
      } => {
        self.resolve_expression(condition);

        let restrictions = self.get_restrictions(condition);
        self.resolve_statement_with_restrictions(then, &restrictions);

        if let Some(otherwise) = otherwise {
          let restrictions = self.inverse_restrictions(&restrictions);
          self.resolve_statement_with_restrictions(otherwise, &restrictions);
        }
      }
      Stmt::While { condition, body } => {
        self.resolve_expression(condition);

        let restrictions = self.get_restrictions(condition);
        self.resolve_statement_with_restrictions(body, &restrictions);
      }
      Stmt::Block { body, .. } => {
        self.begin_scope();
        for statement in body {
          self.resolve_statement(statement);
        }
        self.end_scope();
      }
      Stmt::Expression { expression, .. } => {
        self.resolve_expression(expression);
      }
      Stmt::Comment { .. } => {}
      Stmt::Import { .. } => unimplemented!(),
      Stmt::Return { expression } => {
        if let Some(expression) = expression {
          let expression_type = self.resolve_expression(expression);
          self.assert_type(expression_type, *self.function_stack.last().unwrap(), span);
        }
      }
    }
  }

  fn resolve_expression(&mut self, expression: &Expression<'s>) -> TypeIndex {
    let span = expression.span;

    let type_ = match &expression.expr {
      Expr::Literal { type_, .. } => match type_ {
        LiteralType::String => STRING,
        LiteralType::Number => NUMBER,
        LiteralType::True => TRUE,
        LiteralType::False => FALSE,
        LiteralType::Null => NULL,
      },
      Expr::Group { expression, .. } => self.resolve_expression(expression),
      Expr::Unary {
        operator,
        expression,
        ..
      } => {
        let type_ = self.resolve_expression(expression);
        match operator {
          UnaryOperator::Minus => {
            self.assert_type(type_, NUMBER, span);
            NUMBER
          }
          UnaryOperator::Not => match type_ {
            type_ if self.is_truthy(type_) => FALSE,
            type_ if self.is_falsy(type_) => TRUE,
            _ => BOOLEAN,
          },
        }
      }
      Expr::Binary {
        operator,
        left,
        right,
      } => {
        if let BinaryOperator::Pipeline = operator {
          return self.pipeline(left, right);
        }

        let l = self.resolve_expression(left);
        let r = self.resolve_expression(right);

        match operator {
          BinaryOperator::Plus => {
            self.assert_type(l, NUMBER_OR_STRING, span);
            self.assert_type(r, l, span);
            l
          }
          BinaryOperator::Minus | BinaryOperator::Multiply | BinaryOperator::Divide => {
            self.assert_type(l, NUMBER, span);
            self.assert_type(r, NUMBER, span);
            NUMBER
          }
          BinaryOperator::Equal | BinaryOperator::NotEqual => {
            self.assert_type(r, l, span);
            BOOLEAN
          }
          BinaryOperator::Greater
          | BinaryOperator::Less
          | BinaryOperator::GreaterEqual
          | BinaryOperator::LessEqual => {
            self.assert_type(l, NUMBER_OR_STRING, span);
            self.assert_type(r, l, span);
            BOOLEAN
          }
          BinaryOperator::And => match l {
            type_ if self.is_falsy(type_) => l,
            type_ if self.is_truthy(type_) => r,
            _ => self.union(&[l, r]),
          },
          BinaryOperator::Or => match l {
            type_ if self.is_falsy(type_) => r,
            type_ if self.is_truthy(type_) => l,
            _ => self.union(&[l, r]),
          },
          BinaryOperator::Nullish => {
            if self.matches(NULL, l) {
              let l = self.narrow(l, NULL);
              self.union(&[l, r])
            } else {
              l
            }
          }
          BinaryOperator::Pipeline => unreachable!(),
        }
      }
      Expr::Assignment {
        identifier,
        expression,
      } => {
        let type_ = self.resolve_expression(expression);
        let variable = self.lookup(identifier);

        let variable_type = if let Some(variable) = variable {
          variable.type_
        } else {
          self.error(
            Error::UnknownVariable,
            format!("Variable '{identifier}' is undefined"),
            span,
          )
        };

        if variable_type != NEVER {
          self.assert_type(type_, variable_type, span);
        }

        variable_type
      }
      Expr::Variable { name } => {
        let variable = self.lookup(name);

        if let Some(variable) = variable {
          variable.type_
        } else {
          self.error(
            Error::UnknownVariable,
            format!("Variable '{name}' is undefined"),
            span,
          )
        }
      }
      Expr::Call {
        expression,
        arguments,
      } => {
        let expression_type = self.resolve_expression(expression);
        let arguments = arguments
          .iter()
          .map(|e| self.resolve_expression(e))
          .collect::<Vec<_>>();

        self.call(expression_type, &arguments, span)
      }
      Expr::Function {
        parameters,
        return_type,
        body,
        name,
      } => {
        self.begin_scope();

        let args = parameters
          .iter()
          .map(|parameter| {
            let type_ = self.type_from_annotation(&parameter.type_);
            self.define(parameter.name, type_);
            type_
          })
          .collect::<Vec<_>>();

        let return_type = if let Some(return_type) = &return_type {
          self.type_from_annotation(return_type)
        } else if let Stmt::Return {
          expression: Some(expression),
        } = &body.stmt
        {
          self.resolve_expression(expression)
        } else {
          NULL
        };

        let function = self.add_type(Type::Function(return_type, args));

        if let Some(name) = name {
          // If it has a name, it could be recursive so add it to the enviroment
          self.define(name, function);
        }

        self.function_stack.push(return_type);
        self.resolve_statement(body);
        self.function_stack.pop();

        self.end_scope();
        function
      }
      Expr::Comment { expression, .. } => self.resolve_expression(expression),
    };

    type_
  }

  fn call(&mut self, expression: TypeIndex, arguments: &[TypeIndex], span: Span) -> TypeIndex {
    if let Type::Function(return_type, parameters) = self.types[expression].clone() {
      if parameters.len() != arguments.len() {
        return self.error(
          Error::WrongNumberArguments,
          format!(
            "Expected {} arguments, got {}",
            parameters.len(),
            arguments.len()
          ),
          span,
        );
      }

      for (index, argument) in arguments.iter().enumerate() {
        self.assert_type(*argument, parameters[index], span);
      }

      return_type
    } else {
      let type_name = self.types[expression].get_name(&self.types);
      self.error(
        Error::NotCallable,
        format!("Type '{type_name}' is not callable"),
        span,
      )
    }
  }

  fn pipeline(&mut self, left: &Expression<'s>, right: &Expression<'s>) -> usize {
    let right = if let Expr::Comment { expression, .. } = &right.expr {
      // If right is a comment, unwrap it
      expression
    } else {
      right
    };

    let (expression_type, arguments) = if let Expr::Call {
      expression,
      arguments,
      ..
    } = &right.expr
    {
      let expression = self.resolve_expression(expression);
      let mut arguments = arguments
        .iter()
        .map(|arg| self.resolve_expression(arg))
        .collect::<Vec<_>>();

      arguments.insert(0, self.resolve_expression(left));

      (expression, arguments)
    } else {
      (
        self.resolve_expression(right),
        vec![self.resolve_expression(left)],
      )
    };

    self.call(expression_type, &arguments, right.span)
  }

  fn get_restrictions(&mut self, expression: &Expression<'s>) -> Vec<Restriction<'s>> {
    match &expression.expr {
      Expr::Group { expression } => self.get_restrictions(expression),
      Expr::Binary {
        operator: BinaryOperator::And,
        left,
        right,
      } => {
        let l = self.get_restrictions(left);
        let r = self.get_restrictions(right);

        let mut restrictions = HashMap::<&'s str, TypeIndex>::from_iter(l.into_iter());
        for (name, restriction) in r {
          if let HashMapEntry::Occupied(mut entry) = restrictions.entry(name) {
            let t = self.narrow(restriction, *entry.get());
            entry.insert(t);
          } else {
            restrictions.insert(name, restriction);
          }
        }

        restrictions.into_iter().collect::<Vec<_>>()
      }
      Expr::Binary {
        operator: BinaryOperator::Or,
        left,
        right,
      } => {
        let mut l = self.get_restrictions(left);
        let mut r = self.get_restrictions(right);

        // If different variables are on each side of the OR we can't make a decision
        let all_same_name =
          l.iter().all(|(name, _)| *name == l[0].0) && r.iter().all(|(name, _)| *name == l[0].0);
        if !all_same_name {
          return Vec::new();
        }

        l.append(&mut r);

        vec![(
          l[0].0,
          self.union(
            &l.into_iter()
              .map(|(_, restriction)| restriction)
              .collect::<Vec<_>>(),
          ),
        )]
      }
      Expr::Binary {
        operator: BinaryOperator::Equal,
        left,
        right,
      } => {
        let r = self.resolve_expression(right);

        if let Expr::Variable { name } = &left.expr {
          vec![(name, r)]
        } else {
          Vec::new()
        }
      }
      Expr::Binary {
        operator: BinaryOperator::NotEqual,
        left,
        right,
      } => {
        let l = self.resolve_expression(left);
        let r = self.resolve_expression(right);

        if let Expr::Variable { name } = &left.expr {
          vec![(name, self.narrow(l, r))]
        } else {
          Vec::new()
        }
      }
      _ => Vec::new(),
    }
  }

  fn inverse_restrictions(&mut self, restrictions: &[Restriction<'s>]) -> Vec<Restriction<'s>> {
    restrictions
      .iter()
      .map(|(name, not_type)| {
        let current_type = self
          .variables
          .iter()
          .rfind(|local| local.name == *name)
          .expect("variable to exist as previously found")
          .type_;

        (*name, self.narrow(current_type, *not_type))
      })
      .collect()
  }
}

pub fn typecheck<'s>(source: &'s str, ast: &[Statement<'s>]) -> Vec<Diagnostic> {
  let mut typechecker = Typechecker::new(source);

  for statement in ast {
    typechecker.resolve_statement(statement);
  }

  typechecker.errors
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::{parse, tokenize};

  fn assert_correct(source: &str) {
    let mut ast = parse(source, &tokenize(source)).unwrap();
    let result = typecheck(source, &mut ast);

    assert!(result.is_empty(), "{result:?}");
  }

  fn assert_fails(source: &str) {
    let mut ast = parse(source, &tokenize(source)).unwrap();
    let result = typecheck(source, &mut ast);

    assert!(result.len() > 0, "Test Passes");
  }

  #[test]
  fn literals() {
    assert_correct("let a: string = 'Hello, World!'");
    assert_correct("let a: number = 42");
    assert_correct("let a: boolean = true");
    assert_correct("let a: boolean = false");
    assert_correct("let a: null = null");
  }

  #[test]
  fn declarations() {
    assert_correct("let a = 42\nlet b: number = a\n");
    assert_correct("let a = true\nlet b: boolean = a\n");
    assert_correct("let a = null\nlet b: null = a\n");
    assert_correct("let a\nlet b: null = a\n");
  }

  #[test]
  fn typed_declarations() {
    assert_correct("let a: number = 42\nlet b: number = a\n");
    assert_correct("let a: boolean = true\nlet b: boolean = a\n");
    assert_correct("let a: null = null\nlet b: null = a\n");
    assert_correct("let a: null\nlet b: null = a\n");
    assert_correct("let a: null | null\nlet b: null = a\n");
    assert_correct("let a: null?\nlet b: null = a\n");
    assert_correct("let a: null | number\na = 5\na = null");
    assert_correct("let a: null | number\na = 5 && null\n");
    assert_fails("let a: number = true");
  }

  #[test]
  fn variable_not_defined() {
    assert_fails("a\n");
    assert_fails("let a\nb\n");
  }

  #[test]
  fn grouping() {
    assert_correct("let a: number = (42)");
    assert_correct("let a: boolean = (true)");
    assert_correct("let a: string = ('string')");
  }

  #[test]
  fn unary() {
    assert_correct("let a: number = -42");
    assert_correct("let a: boolean = !true");
    assert_correct("let a: boolean = !false");
    assert_correct("let a: boolean = !'string'");
    assert_correct("let a: boolean = !7");

    assert_fails("-true");
    assert_fails("-'hello'");
    assert_fails("-null");
  }

  #[test]
  fn assignment() {
    assert_correct("let a = 42\nlet b: number = a = 5\n");
    assert_correct("let a = true\nlet b: boolean = a = false\n");
    assert_correct("let a = null\nlet b: null = a = null\n");
    assert_correct("let a\nlet b: null = a = null\n");

    assert_fails("b = 5\n");
    assert_fails("let a = 42\na = false\n");
    assert_fails("let a = false\na = 42\n");
    assert_fails("let a = 'hello'\na = 15\n");
  }

  #[test]
  fn if_and_while() {
    assert_correct(
      "
let a = 5
if (a)
  a = 10
else
  a = 20

let b: number  = a
",
    );
    assert_correct(
      "
let a = 5
while (a)
  a = 0

let b: number = a
",
    );
    assert_fails(
      "
let a = 5
if (b)
  a = 10

let b: number = a
",
    );
  }

  #[test]
  fn binary() {
    assert_correct("let a: number = 5 - 5");
    assert_correct("let a: number = 5 / 5");
    assert_correct("let a: number = 5 * 5");
    assert_correct("let a: boolean = 5 == 5");
    assert_correct("let a: boolean = 5 != 5");
    assert_correct("let a: boolean = 'hello' == 'world'");
    assert_correct("let a: boolean = 'hello' != 'world'");
    assert_correct("let a: boolean = 5 > 5");
    assert_correct("let a: boolean = 5 < 5");
    assert_correct("let a: boolean = 'a' >= 'b'");
  }

  #[test]
  fn plus() {
    assert_correct("let a: number = 5 + 5");
    assert_correct("let a: string = 'hello' + 'world'");
    assert_correct("let a = 'hello' && 5\nlet b: number | string = a + a\n");
    assert_fails("5 + ''");
    assert_fails("'' + 5");
    assert_fails("5 + false");
    assert_fails("null + 5");
    assert_fails("null + true");
  }

  #[test]

  fn minus() {
    assert_fails("'a' - 8");
    assert_fails("8 - 'a'");
    assert_fails("false - null");
  }

  #[test]
  fn comparison() {
    assert_fails("5 == 'a'");
    assert_fails("null != false");
    assert_fails("5 == false");
  }

  #[test]
  fn nullish_coelesing() {
    assert_correct("let a: boolean = null ?? false");
    assert_correct("let a: string = 'hello' ?? null");
    assert_correct("let a: number = 5 ?? ''");
    assert_correct("let a: null = null ?? null");
    assert_correct("let a: number = 5 ?? 6");
    assert_correct("let a: number  = 5 ?? null");
    assert_correct("let a: number = null ?? 5");
    assert_correct("let a: number = 5 ?? false");
  }

  #[test]
  fn and() {
    assert_correct("let a: number = 5 && 6");
    assert_correct("let a: number | null = 5 && null");
    assert_correct("let a: number? = 5 && null");
    assert_correct("let a: null = null && 5");
    assert_correct("let a: number | boolean = 5 && false");
    assert_correct("let a: boolean  = false && 5");
  }

  #[test]
  fn or() {
    assert_correct("let a: number = 5 || 6");
    assert_correct("let a: number | null = 5 || null");
    assert_correct("let a: number = null || 5");
    assert_correct("let a: number | boolean = 5 || false");
    assert_correct("let a: number  = false || 5");
  }

  #[test]
  fn call_not_callable() {
    assert_fails("5()");
    assert_fails("'hello'()");
    assert_fails("true()");
    assert_fails("null()");
  }

  #[test]
  fn call() {
    assert_correct("let a: null = (() => null)()");
    assert_correct("let b: string = (() => 'hello')()");
    assert_correct("let c: number = ((a: number, b: number) => a + b)(7, 8)");
    assert_correct(
      "
let not = (x: any) => !x
let a: boolean = not(true)
let b: boolean = not(false)
let c: boolean = not(null)
let d: boolean = not(3.5)
      ",
    );
    assert_fails("((a: number, b: number) => a + b)(7)");
    assert_fails("((a: number, b: number) => a + b)(7, 8, 9)");
    assert_fails("((a: number, b: number) => a + b)(7, false)");
    assert_fails("((a: number, b: number) => a + b)(7, null)");
  }

  #[test]
  fn functions() {
    assert_correct("let func: (number, number) -> number = (a: number, b: number) => a + b");
    assert_correct("let a: ((any) -> null) | ((any) -> string) = print");
    assert_correct("let a: ((any) -> null) | ((any) -> string) = type");
    assert_correct("let p: (any) -> null = print\nlet t: (any) -> string = type\n");
    assert_correct(
      "let func: (number | string) -> number | string = (a: number | string | boolean) => 7",
    );
    assert_fails("let func: (number, string) -> number  = (a: number, b: string) => a || b");
  }

  #[test]
  fn recursive() {
    assert_correct(
      "
let fib_recursive = (n: number) -> number
  if (n <= 2)
    if (n == 0) return 0
    return n - 1
  else return fib_recursive(n - 1) + fib_recursive(n - 2)

let a: number = fib_recursive(25)
",
    );
  }

  #[test]
  #[should_panic]
  fn corecursive() {
    assert_correct(
      "
let a = (n: number) -> number
  if (n > 0)
    return b(n)
  return n

let b = (n: number) -> number
  return a(n-1)

let c: number = b(5)
",
    );
  }

  #[test]
  #[should_panic]
  fn imports() {
    assert_correct("from string import {{ trim }}\nlet a: (string) -> string = trim");
    assert_correct("from string import {{ toNumber }}\nlet a: (string) -> number? = toNumber");
    assert_correct(
      "from string import {{ includes }}\nlet a: (string, string) -> boolean = includes",
    );

    assert_correct("from maths import {{ pow }}\nlet a: (number, number) -> number = pow");
    assert_correct("from maths import {{ sin }}\nlet a: (number) -> number = sin");
  }

  #[test]
  fn returns() {
    assert_correct(
      "
let numbers = (n: number) -> number
  if (n <= 2)
    if (n == 0) return 0
    return n - 1
  else return n * 5

let a: number = numbers(25)
",
    );
    assert_correct(
      "
let x = (n: number) ->
  let a = 7

let a: null = x(6)
",
    );
    assert_fails(
      "
let numbers = (n: number) -> number
  if (n <= 2)
    if (n == 0) return 0
    return n - 1
  else return n * 5 || 'hello'

numbers(25)
  ",
    );
  }

  #[test]
  fn pipeline() {
    assert_correct(
      "
let add_one = (a: number) => a + 1

let a:number = 3 >> add_one
",
    );
    assert_correct(
      "
let add_one = (a: number) => a + 1

let a:number = 3 >> add_one()
",
    );
    assert_correct(
      "
let add = (a: number, b: number) => a + b
let multiply = (a: number, b: number) => a * b

let a:number = 3 >> add(4) >> multiply(5)
    ",
    );
  }

  #[test]
  fn redefined_variables() {
    assert_correct(
      "
let a = false
  let a = 5
  a = -a
",
    );
  }

  mod narrowing {
    use super::*;

    #[test]
    fn not_equals() {
      assert_correct(
        "
let func = (a: number?) ->
  if (a != null) -a
",
      );
    }

    #[test]
    fn equals() {
      assert_correct(
        "
let boolean = (b: boolean) => b
let func = (a: boolean?) ->
  if (a == true) boolean(a)

",
      );
    }

    #[test]
    fn or() {
      assert_correct(
        "
let boolean = (b: boolean) => b
let func = (a: boolean?) ->
  if (a == true || a == false) boolean(a)

",
      );
    }

    #[test]
    fn and() {
      assert_correct(
        "
let boolean = (b: boolean) => b
let func = (a: boolean?) ->
  if (a == true && a == false) boolean(a)

",
      );
    }

    #[test]
    fn multiple_and() {
      assert_correct(
        "
let boolean = (b: boolean) => b
let func = (a: boolean?, b: boolean?) ->
  if (a == true && b == false)
    boolean(a)
    boolean(b)
",
      );
    }

    #[test]
    fn multiple_or() {
      assert_fails(
        "
let boolean = (b: boolean) => b

let func = (a: boolean?, b: boolean?) ->
  if (a == true || b == false)
    boolean(a)
    boolean(b)

func(true, false)
",
      );
    }

    #[test]
    fn else_() {
      assert_correct(
        "
let boolean = (b: boolean) => b
let n = (n: null) => null

let func = (a: boolean?) ->
  if (a == true || a == false) boolean(a)
  else n(a)
",
      );
    }
  }
}
