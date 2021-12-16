use super::rule::{Rule, RuleResult};
use crate::ast::{Expression, LiteralValue, Statement, Visitor};
use crate::token::Token;

pub struct NoNegativeZero {
  issues: Vec<Token>,
}

impl Rule for NoNegativeZero {
  fn check(ast: &[Statement]) -> RuleResult {
    let mut visitor = NoNegativeZero { issues: Vec::new() };
    visitor.visit(ast);

    RuleResult {
      name: "No Negative Zero",
      message: "Negative zero is unnecessary as 0 == -0",
      issues: visitor.issues,
    }
  }
}

impl Visitor for NoNegativeZero {
  fn exit_expression(&mut self, expression: &Expression) {
    if let Expression::Unary {
      expression, token, ..
    } = expression
    {
      if let Expression::Literal {
        value: LiteralValue::Number(num),
        ..
      } = expression.as_ref()
      {
        if *num == 0.0 {
          self.issues.push(*token);
        }
      }
    }
  }
}
