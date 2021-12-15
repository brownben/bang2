use super::rule::{Rule, RuleResult, Visitor};
use crate::ast::{BinaryOperator, Expression, Statement};
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
  fn visit_expression(&mut self, expression: &Expression) {
    match expression {
      Expression::Assignment { expression, .. } => self.visit_expression(expression),
      Expression::Binary {
        left,
        right,
        operator,
        token,
        ..
      } => {
        if let BinaryOperator::EqualEqual | BinaryOperator::BangEqual = operator {
          if let Expression::Variable { .. } = &**right {
            if let Expression::Literal { .. } = &**left {
              self.issues.push(*token);
            }
          }
        }

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
      Expression::Unary { expression, .. } => self.visit_expression(expression),
      Expression::Variable { .. } => {}
    }
  }
}
