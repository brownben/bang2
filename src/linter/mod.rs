use crate::ast::Statement;

mod rule;
use rule::Rule;
pub use rule::RuleResult;

mod no_constant_condition;
mod no_negative_zero;
mod no_self_assign;
mod no_unreachable;
mod no_yoda_equality_check;

pub fn lint(ast: &[Statement]) -> Vec<RuleResult> {
  let mut results = vec![
    no_constant_condition::NoConstantCondition::check(ast),
    no_negative_zero::NoNegativeZero::check(ast),
    no_self_assign::NoSelfAssign::check(ast),
    no_unreachable::NoUnreachable::check(ast),
    no_yoda_equality_check::NoYodaEqualityCheck::check(ast),
  ];

  results.retain(|r| !r.issues.is_empty());
  results
}
