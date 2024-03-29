mod constant_condition;
mod side_effects;
mod unused_variables;
mod visitor;

use bang_syntax::{
  ast::{
    expression::{operators::Binary as BinaryOperator, Expr, Expression},
    statement::{Statement, Stmt},
    Span,
  },
  Diagnostic as ParserDiagnostic, LineNumber, Parser,
};
use std::{error, fmt};
use visitor::Visitor;

trait LintRule {
  fn check(source: &str, ast: &[Statement]) -> Diagnostic;
}

#[derive(Debug)]
pub struct Diagnostic {
  pub title: String,
  pub message: String,
  pub spans: Vec<Span>,
  pub lines: Vec<LineNumber>,
}
impl fmt::Display for Diagnostic {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "Lint Warning: {}\n\t{}\nat lines {}",
      self.title,
      self.message,
      self
        .lines
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
    )
  }
}
impl error::Error for Diagnostic {}
impl From<ParserDiagnostic> for Diagnostic {
  fn from(diagnostic: ParserDiagnostic) -> Self {
    Self {
      title: diagnostic.title,
      message: diagnostic.message,
      spans: vec![diagnostic.span],
      lines: vec![diagnostic.line],
    }
  }
}

macro_rules! lint_rule {
  {
    name: $rule_name:ident;
    title: $title:expr;
    message: $message:expr;
    data: $type:ty;
    visitor: $visitor:tt
  } => {
    pub struct $rule_name {
      issues: Vec<Span>,
      data: $type
    }
    impl Default for $rule_name {
      fn default() -> Self {
        Self {
          issues: Vec::new(),
          data: Default::default()
        }
      }
    }
    lint_rule! { trait $rule_name; $title; $message; $visitor }
  };

  {
    name: $rule_name:ident;
    title: $title:expr;
    message: $message:expr;
    visitor: $visitor:tt
  } => {
    pub struct $rule_name {
      issues: Vec<Span>,
    }
    impl Default for $rule_name {
      fn default() -> Self {
        Self { issues: Vec::new() }
      }
    }
    lint_rule! { trait $rule_name; $title; $message; $visitor }
  };

  { trait $rule_name:ident; $title:expr; $message:expr; $visitor:tt } => {
    impl LintRule for $rule_name {
      fn check(source: &str, ast: &[Statement]) -> Diagnostic {
        let mut visitor = Self::default();
        visitor.visit(ast);

        Diagnostic {
          title: $title.to_string(),
          message: $message.to_string(),
          lines: visitor.issues.iter().map(|span| span.get_line_number(source)).collect(),
          spans: visitor.issues,
        }
      }
    }
    impl Visitor for $rule_name $visitor
  }
}
pub(crate) use lint_rule;

lint_rule! {
  name: NoYodaEquality;
  title: "No Yoda Equality";
  message: "It is clearer to have the variable first then the value to compare to";
  visitor: {
    fn exit_expression(&mut self, expression: &Expression) {
      if let Expr::Binary { left, right, operator, ..} = &expression.expr
        && let BinaryOperator::Equal | BinaryOperator::NotEqual = operator
        && let Expr::Variable { .. } = right.expr
        && let Expr::Literal { .. } = left.expr
      {
        self.issues.push(expression.span);
      }
    }
  }
}

lint_rule! {
  name: NoNegativeZero;
  title: "No Negative Zero";
  message: "Negative zero is unnecessary as 0 == -0";
  visitor: {
    fn exit_expression(&mut self, expression: &Expression) {
      if let Expr::Unary { expression, .. } = &expression.expr
        && let Expr::Literal { value,  .. } = &expression.expr
        && Parser::number(value) == 0.0
      {
        self.issues.push(expression.span);
      }
    }
  }
}

lint_rule! {
  name: NoSelfAssign;
  title: "No Self Assign";
  message: "Assigning a variable to itself is unnecessary";
  visitor: {
    fn exit_expression(&mut self, expression: &Expression) {
      if let Expr::Assignment {
        identifier,
        expression,
        ..
      } = &expression.expr
      {
        if let Expr::Variable { name, .. } = &expression.expr
          && identifier == name
        {
          self.issues.push(expression.span);
        }
      }
    }
  }
}

lint_rule! {
  name: NoUnreachable;
  title: "No Unreachable Code";
  message: "Code after a return can never be executed";
  visitor: {
    fn exit_statement(&mut self, statement: &Statement) {
      if let Stmt::Block { body, .. } = &statement.stmt {
        let mut seen_return = false;
        for statement in body {
          if seen_return {
            self.issues.push(statement.span);
            break;
          }

          if let Stmt::Return { .. } = statement.stmt {
            seen_return = true;
          }
        }
      }
    }
  }
}

pub fn lint(source: &str, ast: &[Statement]) -> Vec<Diagnostic> {
  let mut results = vec![
    NoYodaEquality::check(source, ast),
    NoNegativeZero::check(source, ast),
    NoSelfAssign::check(source, ast),
    NoUnreachable::check(source, ast),
    constant_condition::NoConstantCondition::check(source, ast),
    side_effects::NoSideEffectInIndex::check(source, ast),
    unused_variables::NoUnusedVariables::check(source, ast),
  ];

  results.retain(|r| !r.lines.is_empty());
  results
}
