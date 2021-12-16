use super::rule::{Rule, RuleResult};
use crate::ast::{BinaryOperator, Expression, Statement, Visitor};
use crate::token::Token;

pub struct NoYodaEqualityCheck {
  issues: Vec<Token>,
}

impl Rule for NoYodaEqualityCheck {
  fn check(ast: &[Statement]) -> RuleResult {
    let mut visitor = NoYodaEqualityCheck { issues: Vec::new() };
    visitor.visit(ast);

    RuleResult {
      name: "No Yoda Equality",
      message: "It is clearer to have the variable first then the value to compare to",
      issues: visitor.issues,
    }
  }
}

impl Visitor for NoYodaEqualityCheck {
  fn exit_expression(&mut self, expression: &Expression) {
    if let Expression::Binary {
      left,
      right,
      operator,
      token,
      ..
    } = expression
    {
      if let BinaryOperator::EqualEqual | BinaryOperator::BangEqual = operator {
        if let Expression::Variable { .. } = right.as_ref() {
          if let Expression::Literal { .. } = left.as_ref() {
            self.issues.push(*token);
          }
        }
      }
    }
  }
}
