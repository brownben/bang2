use crate::ast::Statement;
use crate::token::Token;

pub struct RuleResult {
  pub name: &'static str,
  pub message: &'static str,
  pub issues: Vec<Token>,
}

pub trait Rule {
  fn check(ast: &[Statement]) -> RuleResult;
}
