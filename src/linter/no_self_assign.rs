use super::rule::{Rule, RuleResult, Visitor};
use crate::ast::{Expression, Statement};
use crate::token::Token;

pub struct NoSelfAssign {
  issues: Vec<Token>,
}

impl Rule for NoSelfAssign {
  fn check(ast: &[Statement]) -> RuleResult {
    let mut visitor = NoSelfAssign { issues: Vec::new() };
    visitor.visit(ast);

    RuleResult {
      name: "No Self Assign",
      message: "Assigning a variable to itself is unnecessary",
      issues: visitor.issues,
    }
  }
}

impl Visitor for NoSelfAssign {
  fn visit_expression(&mut self, expression: &Expression) {
    match expression {
      Expression::Assignment {
        identifier,
        variable_name: name,
        expression,
        ..
      } => {
        if let Expression::Variable { variable_name, .. } = &**expression {
          if name == variable_name {
            self.issues.push(*identifier);
          }
        } else {
          self.visit_expression(expression)
        }
      }
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
      Expression::Unary { expression, .. } => self.visit_expression(expression),
      Expression::Variable { .. } => {}
    }
  }
}
