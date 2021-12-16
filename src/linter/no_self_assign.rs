use super::rule::{Rule, RuleResult};
use crate::ast::{Expression, Statement, Visitor};
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
  fn exit_expression(&mut self, expression: &Expression) {
    if let Expression::Assignment {
      identifier,
      variable_name: name,
      expression,
      ..
    } = expression
    {
      if let Expression::Variable { variable_name, .. } = expression.as_ref() {
        if name == variable_name {
          self.issues.push(*identifier);
        }
      }
    }
  }
}
