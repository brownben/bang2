// Known Problems in Typechecker:
// - Doesn't support imports
// - Doesn't support corecursion, or accessing globals before they are defined

use crate::{
  ast::{
    BinaryOperator, Expr, Expression, LiteralType, Span, Statement, Stmt, Type as TypeItem,
    TypeExpression, UnaryOperator,
  },
  Diagnostic,
};

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

const NULL: TypeIndex = 0;
const NUMBER: TypeIndex = 1;
const STRING: TypeIndex = 2;
const TRUE: TypeIndex = 3;
const FALSE: TypeIndex = 4;
const BOOLEAN: TypeIndex = 5;
const ANY: TypeIndex = 6;
const NEVER: TypeIndex = 7;
const NUMBER_OR_STRING: TypeIndex = 8;

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
        let mut names = args
          .iter()
          .map(|t| types[*t].get_name(types))
          .collect::<Vec<_>>();
        names.sort();
        format!(
          "({}) -> {}",
          types[*return_type].get_name(types),
          names.join(", ")
        )
      }
    }
  }
}

struct Typechecker<'s> {
  source: &'s str,

  functions: Vec<TypeIndex>,
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
      functions: Vec::new(),
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
            .all(|(a, b)| self.matches(*a, *b))
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
      .iter()
      .filter(|t| !self.matches(**t, b))
      .cloned()
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

  fn resolve_statement(&mut self, statement: &mut Statement<'s>) {
    let span = statement.span;

    match &mut statement.stmt {
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
        self.resolve_statement(then);

        if let Some(otherwise) = otherwise {
          self.resolve_statement(otherwise);
        }
      }
      Stmt::While { condition, body } => {
        self.resolve_expression(condition);
        self.resolve_statement(body);
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
          self.assert_type(expression_type, *self.functions.last().unwrap(), span);
        }
      }
    }
  }

  fn resolve_expression(&mut self, expression: &mut Expression<'s>) -> TypeIndex {
    let span = expression.span;

    let type_ = match &mut expression.expr {
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
          let type_ = self.pipeline(left, right);
          expression.type_ = Some(type_);
          return type_;
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
            self.assert_type(l, r, span);
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
        let variable = self
          .variables
          .iter()
          .find(|local| local.name == *identifier);

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
        let variable = self.variables.iter().find(|local| local.name == *name);

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
          .iter_mut()
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
        } = &mut body.stmt
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

        self.functions.push(return_type);
        self.resolve_statement(body);
        self.functions.pop();

        self.end_scope();
        function
      }
      Expr::Comment { expression, .. } => self.resolve_expression(expression),
    };

    expression.type_ = Some(type_);
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

  fn pipeline(&mut self, left: &mut Expression<'s>, right: &mut Expression<'s>) -> usize {
    let right = if let Expr::Comment { expression, .. } = &mut right.expr {
      // If right is a comment, unwrap it
      expression
    } else {
      right
    };

    let (expression_type, arguments) = if let Expr::Call {
      expression,
      arguments,
      ..
    } = &mut right.expr
    {
      let expression = self.resolve_expression(expression);
      let mut arguments = arguments
        .iter_mut()
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
}

pub fn typecheck<'s>(source: &'s str, ast: &mut [Statement<'s>]) -> Vec<Diagnostic> {
  let mut typechecker = Typechecker::new(source);

  for statement in ast {
    typechecker.resolve_statement(statement);
  }

  typechecker.errors
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::{ast::Span, parse, parser::parse_type, tokenize};

  fn assert_expression_type(source: &str, expected: &str) {
    let mut ast = parse(source, &tokenize(source)).unwrap();
    let mut typechecker = Typechecker::new(source);
    let expected_type = {
      let type_ast = parse_type(expected, &tokenize(expected)).unwrap();
      typechecker.type_from_annotation(&type_ast)
    };

    for statement in &mut ast {
      typechecker.resolve_statement(statement);
    }

    if let Stmt::Expression { expression } = &ast.last().unwrap().stmt {
      let type_ = &expression.type_.unwrap();
      typechecker.assert_type(*type_, expected_type, Span { start: 0, end: 0 });
    } else {
      panic!("Expected Expression");
    }

    if !typechecker.errors.is_empty() {
      panic!("Types Don't Match");
    }
  }

  fn typecheck(source: &str) -> Result<(), ()> {
    let mut ast = parse(source, &tokenize(source)).unwrap();
    let mut typechecker = Typechecker::new(source);

    for statement in &mut ast {
      typechecker.resolve_statement(statement);
    }

    if typechecker.errors.is_empty() {
      Ok(())
    } else {
      Err(())
    }
  }

  #[test]
  fn literals() {
    assert_expression_type("'Hello, World!'", "string");
    assert_expression_type("42", "number");
    assert_expression_type("true", "boolean");
    assert_expression_type("false", "boolean");
    assert_expression_type("null", "null");
  }

  #[test]
  fn declarations() {
    assert_expression_type("let a = 42\na\n", "number");
    assert_expression_type("let a = true\na\n", "boolean");
    assert_expression_type("let a = null\na\n", "null");
    assert_expression_type("let a\na\n", "null");
  }

  #[test]
  fn typed_declarations() {
    assert_expression_type("let a: number = 42\na\n", "number");
    assert_expression_type("let a: boolean = true\na\n", "boolean");
    assert_expression_type("let a: null = null\na\n", "null");
    assert_expression_type("let a: null\na\n", "null");
    assert_expression_type("let a: null | null\na\n", "null");
    assert_expression_type("let a: null?\na\n", "null");
    assert!(typecheck("let a: null | number\na = 5\na = null").is_ok());
    assert!(typecheck("let a: null | number\na = 5 && null\n").is_ok());

    let result = typecheck("let a: number = true");
    assert!(result.is_err())
  }

  #[test]
  fn variable_not_defined() {
    let result = typecheck("a\n");
    assert!(result.is_err());

    let result = typecheck("let a\nb\n");
    assert!(result.is_err());
  }

  #[test]
  fn grouping() {
    assert_expression_type("(42)", "number");
    assert_expression_type("(true)", "boolean");
    assert_expression_type("('string')", "string");
  }

  #[test]
  fn unary() {
    assert_expression_type("-42", "number");
    assert_expression_type("!true", "boolean");
    assert_expression_type("!false", "boolean");
    assert_expression_type("!'string'", "boolean");
    assert_expression_type("!7", "boolean");

    let result = typecheck("-true");
    assert!(result.is_err());

    let result = typecheck("-'hello'");
    assert!(result.is_err());

    let result = typecheck("-null");
    assert!(result.is_err());
  }

  #[test]
  fn assignment() {
    assert_expression_type("let a = 42\na = 5\n", "number");
    assert_expression_type("let a = true\na = false\n", "boolean");
    assert_expression_type("let a = null\na = null\n", "null");
    assert_expression_type("let a\na = null\n", "null");

    let result = typecheck("b = 5\n");
    assert!(result.is_err());

    let result = typecheck("let a = 42\na = false\n");
    assert!(result.is_err());

    let result = typecheck("let a = false\na = 42\n");
    assert!(result.is_err());

    let result = typecheck("let a = 'hello'\na = 15\n");
    assert!(result.is_err());
  }

  #[test]
  fn if_and_while() {
    assert_expression_type(
      "
let a = 5
if (a)
  a = 10
else
  a = 20

a
",
      "number",
    );
    assert_expression_type(
      "
let a = 5
while (a)
  a = 0

a
",
      "number",
    );

    let result = typecheck(
      "
let a = 5
if (b)
  a = 10

let b: number = a
",
    );
    assert!(result.is_err());
  }

  #[test]
  fn binary() {
    assert_expression_type("5 - 5", "number");
    assert_expression_type("5 / 5", "number");
    assert_expression_type("5 * 5", "number");
    assert_expression_type("5 == 5", "boolean");
    assert_expression_type("5 != 5", "boolean");
    assert_expression_type("'hello' == 'world'", "boolean");
    assert_expression_type("'hello' != 'world'", "boolean");
    assert_expression_type("5 > 5", "boolean");
    assert_expression_type("5 < 5", "boolean");
    assert_expression_type("'a' >= 'b'", "boolean");
  }

  #[test]
  fn plus() {
    assert_expression_type("5 + 5", "number");
    assert_expression_type("'hello' + 'world'", "string");
    assert!(typecheck("let a = 'hello' && 5\nlet b: number | string = a + a\n").is_ok());

    let result = typecheck("5 + ''");
    assert!(result.is_err());

    let result = typecheck("'' + 5");
    assert!(result.is_err());

    let result = typecheck("5 + false");
    assert!(result.is_err());

    let result = typecheck("null + 5");
    assert!(result.is_err());

    let result = typecheck("null + true");
    assert!(result.is_err());
  }

  #[test]

  fn minus() {
    let result = typecheck("'a' - 8");
    assert!(result.is_err());

    let result = typecheck("8 - 'a'");
    assert!(result.is_err());

    let result = typecheck("false - null");
    assert!(result.is_err());
  }

  #[test]
  fn comparison() {
    let result = typecheck("5 == 'a'");
    assert!(result.is_err());

    let result = typecheck("null != false");
    assert!(result.is_err());

    let result = typecheck("5 == false");
    assert!(result.is_err());
  }

  #[test]
  fn nullish_coelesing() {
    assert_expression_type("null ?? false", "boolean");
    assert_expression_type("'hello' ?? null", "string");
    assert_expression_type("5 ?? ''", "number");
    assert_expression_type("null ?? null", "null");
    assert!(typecheck("let a: number = 5 ?? 6").is_ok());
    assert!(typecheck("let a: number  = 5 ?? null").is_ok());
    assert!(typecheck("let a: number = null ?? 5").is_ok());
    assert!(typecheck("let a: number = 5 ?? false").is_ok());
  }

  #[test]
  fn and() {
    assert!(typecheck("let a: number = 5 && 6").is_ok());
    assert!(typecheck("let a: number | null = 5 && null").is_ok());
    assert!(typecheck("let a: number? = 5 && null").is_ok());
    assert!(typecheck("let a: null = null && 5").is_ok());
    assert!(typecheck("let a: number | boolean = 5 && false").is_ok());
    assert!(typecheck("let a: boolean  = false && 5").is_ok());
  }

  #[test]
  fn or() {
    assert!(typecheck("let a: number = 5 || 6").is_ok());
    assert!(typecheck("let a: number | null = 5 || null").is_ok());
    assert!(typecheck("let a: number = null || 5").is_ok());
    assert!(typecheck("let a: number | boolean = 5 || false").is_ok());
    assert!(typecheck("let a: number  = false || 5").is_ok());
  }

  #[test]
  fn call_not_callable() {
    let result = typecheck("5()");
    assert!(result.is_err());

    let result = typecheck("'hello'()");
    assert!(result.is_err());

    let result = typecheck("true()");
    assert!(result.is_err());

    let result = typecheck("null()");
    assert!(result.is_err());
  }

  #[test]
  fn call() {
    assert_expression_type("(() => null)()", "null");
    assert_expression_type("(() => 'hello')()", "string");
    assert_expression_type("((a: number, b: number) => a + b)(7, 8)", "number");
    assert!(typecheck(
      "
let not = (x: any) => !x
let a: boolean = not(true)
let b: boolean = not(false)
let c: boolean = not(null)
let d: boolean = not(3.5)
      ",
    )
    .is_ok());

    let result = typecheck("((a: number, b: number) => a + b)(7)");
    assert!(result.is_err());

    let result = typecheck("((a: number, b: number) => a + b)(7, 8, 9)");
    assert!(result.is_err());

    let result = typecheck("((a: number, b: number) => a + b)(7, false)");
    assert!(result.is_err());

    let result = typecheck("((a: number, b: number) => a + b)(7, null)");
    assert!(result.is_err());
  }

  #[test]
  fn functions() {
    assert!(
      typecheck("let func: (number, number) -> number = (a: number, b: number) => a + b").is_ok()
    );

    assert_expression_type("print", "((any) -> null) | ((any) -> string)");
    assert_expression_type("type", "((any) -> null) | ((any) -> string)");
    assert!(typecheck("let p: (any) -> null = print\nlet t: (any) -> string = type\n",).is_ok());
  }

  #[test]
  fn recursive() {
    assert_expression_type(
      "
let fib_recursive = (n: number) -> number
  if (n <= 2)
    if (n == 0) return 0
    return n - 1
  else return fib_recursive(n - 1) + fib_recursive(n - 2)

fib_recursive(25)
",
      "number",
    );
  }

  #[test]
  fn returns() {
    assert_expression_type(
      "
let numbers = (n: number) -> number
  if (n <= 2)
    if (n == 0) return 0
    return n - 1
  else return n * 5

numbers(25)
",
      "number",
    );

    assert_expression_type(
      "
let x = (n: number) ->
  let a = 7

x(6)
",
      "null",
    );

    let result = typecheck(
      "
let numbers = (n: number) -> number
  if (n <= 2)
    if (n == 0) return 0
    return n - 1
  else return n * 5 || 'hello'

numbers(25)
  ",
    );
    assert!(result.is_err());
  }

  #[test]
  fn pipeline() {
    assert_expression_type(
      "
let add_one = (a: number) => a + 1

3 >> add_one
",
      "number",
    );
    assert_expression_type(
      "
let add_one = (a: number) => a + 1

3 >> add_one()
",
      "number",
    );
    assert_expression_type(
      "
let add = (a: number, b: number) => a + b
let multiply = (a: number, b: number) => a * b

3 >> add(4) >> multiply(5)
    ",
      "number",
    );
  }
}
