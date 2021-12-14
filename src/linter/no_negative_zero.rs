use super::rule::{Rule, RuleResult, Visitor};
use crate::ast::{Expression, LiteralValue, Statement};
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
  fn visit_expression(&mut self, expression: &Expression) {
    match expression {
      Expression::Unary {
        expression,
        operator,
        ..
      } => {
        if let Expression::Literal { value, .. } = expression.as_ref() {
          if let LiteralValue::Number(num) = value {
            if *num == 0.0 {
              self.issues.push(*operator);
            }
          }
        } else {
          self.visit_expression(expression)
        }
      }
      Expression::Assignment { expression, .. } => self.visit_expression(expression),
      Expression::Binary { left, right, .. } => {
        self.visit_expression(left);
        self.visit_expression(right);
      }
      Expression::Call {
        expression,
        arguments,
        ..
      } => {
        self.visit_expression(expression);
        arguments.iter().for_each(|arg| self.visit_expression(arg));
      }
      Expression::Group { expression, .. } => self.visit_expression(expression),
      Expression::Literal { .. } => {}
      Expression::Variable { .. } => {}
    }
  }
}
