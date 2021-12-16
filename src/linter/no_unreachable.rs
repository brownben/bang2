use super::rule::{Rule, RuleResult};
use crate::ast::{Statement, Visitor};
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
  fn exit_statement(&mut self, statement: &Statement) {
    if let Statement::Block { body, .. } = statement {
      let mut seen_return: Option<Token> = None;
      for statement in body {
        if let Some(token) = seen_return {
          self.issues.push(token);
          break;
        }

        if let Statement::Return { token, .. } = statement {
          seen_return = Some(*token);
        }
      }
    }
  }
}
