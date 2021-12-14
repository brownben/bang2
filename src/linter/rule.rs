use crate::ast::{Statement, Expression};
use crate::token::Token;

pub struct RuleResult {
  pub name: &'static str,
  pub message: &'static str,
  pub issues: Vec<Token>,
}

pub trait Rule {
  fn check(ast: &[Statement]) -> RuleResult;
}

pub trait Visitor {
  fn visit(&mut self, statements: &[Statement]) {
    statements.iter().for_each(|s| self.visit_statement(s));
  }

  fn visit_statement(&mut self, statement: &Statement) {
    match statement {
      Statement::Block { body, .. } => body.iter().for_each(|s| self.visit_statement(s)),
      Statement::Declaration { expression, .. } => {
        if let Some(expression) = &*expression {
          self.visit_expression(expression)
        }
      }
      Statement::Expression { expression, .. } => self.visit_expression(expression),
      Statement::Function { body, .. } => self.visit_statement(body),
      Statement::If {
        condition,
        then,
        otherwise,
        ..
      } => {
        self.visit_expression(condition);
        self.visit_statement(then);
        if let Some(otherwise) = &*otherwise {
          self.visit_statement(otherwise.as_ref())
        }
      }
      Statement::Return { expression, .. } => {
        if let Some(expression) = expression {
          self.visit_expression(expression)
        }
      }
      Statement::While {
        condition, body, ..
      } => {
        self.visit_expression(condition);
        self.visit_statement(body)
      }
    }
  }

  fn visit_expression(&mut self, expression: &Expression) {
    match expression {
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
      Expression::Unary { expression, .. } => self.visit_expression(expression),
      Expression::Variable { .. } => {}
    }
  }
}
