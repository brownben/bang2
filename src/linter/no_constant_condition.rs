use super::rule::{Rule, RuleResult, Visitor};
use crate::ast::{Expression, Statement};
use crate::token::Token;

pub struct NoConstantCondition {
  issues: Vec<Token>,
}

impl Rule for NoConstantCondition {
  fn check(ast: &[Statement]) -> RuleResult {
    let mut visitor = NoConstantCondition { issues: Vec::new() };
    visitor.visit(ast);

    RuleResult {
      name: "No Constant Conditions",
      message: "The control flow could be removed, as the condition is always true or false",
      issues: visitor.issues,
    }
  }
}

fn has_variable_access(expression: &Expression) -> bool {
  match expression {
    Expression::Variable { .. } => true,
    Expression::Literal { .. } => false,
    Expression::Assignment { expression, .. } => has_variable_access(expression),
    Expression::Group { expression, .. } => has_variable_access(expression),
    Expression::Unary { expression, .. } => has_variable_access(expression),
    Expression::Binary { left, right, .. } => {
      has_variable_access(left) || has_variable_access(right)
    }
    Expression::Call {
      expression,
      arguments,
      ..
    } => has_variable_access(expression) || arguments.iter().any(has_variable_access),
  }
}

impl Visitor for NoConstantCondition {
  fn visit_statement(&mut self, statement: &Statement) {
    match statement {
      Statement::If {
        if_token,
        condition,
        then,
        otherwise,
        ..
      } => {
        if !has_variable_access(condition) {
          self.issues.push(*if_token);
        }
        self.visit_statement(then);
        if let Some(otherwise) = &*otherwise {
          self.visit_statement(otherwise.as_ref())
        }
      }
      Statement::While {
        token,
        condition,
        body,
        ..
      } => {
        if !has_variable_access(condition) {
          self.issues.push(*token);
        }
        self.visit_statement(body)
      }
      Statement::Block { body, .. } => body.iter().for_each(|s| self.visit_statement(s)),
      Statement::Declaration { .. } => {}
      Statement::Expression { .. } => {}
      Statement::Function { body, .. } => self.visit_statement(body),
      Statement::Return { .. } => {}
    }
  }
}
