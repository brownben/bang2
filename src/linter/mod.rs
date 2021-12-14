use crate::ast::Statement;

mod rule;

mod no_negative_zero;
mod no_unreachable;

use rule::Rule;
pub use rule::RuleResult;

pub fn lint(ast: &[Statement]) -> Vec<RuleResult> {
  let mut results = vec![
    no_negative_zero::NoNegativeZero::check(ast),
    no_unreachable::NoUnreachable::check(ast),
  ];

  results.retain(|r| !r.issues.is_empty());
  results
}
