use super::rule::{Rule, RuleResult, Visitor};
use crate::ast::Statement;
use crate::token::Token;

pub struct NoUnreachable {
  issues: Vec<Token>,
}

impl Rule for NoUnreachable {
  fn check(ast: &[Statement]) -> RuleResult {
    let mut visitor = NoUnreachable { issues: Vec::new() };

    visitor.visit(ast);

    RuleResult {
      name: "No Unreachable Code",
      message: "Code after a return can never be executed",
      issues: visitor.issues,
    }
  }
}

impl Visitor for NoUnreachable {
  fn visit_statement(&mut self, statement: &Statement) {
    match statement {
      Statement::Block { body, .. } => {
        let mut seen_return: Option<Token> = None;
        for statement in body {
          if let Some(token) = seen_return {
            self.issues.push(token);
            break;
          }

          if let Statement::Return { token, .. } = statement {
            seen_return = Some(*token);
          } else {
            self.visit_statement(statement);
          }
        }
      }
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
}
