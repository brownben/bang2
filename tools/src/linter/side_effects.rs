use super::{lint_rule, Diagnostic, LintRule, Visitor};
use bang_syntax::ast::{
  expression::{Expr, Expression},
  statement::Statement,
  Span,
};

fn has_possible_side_effect(expression: &Expr) -> bool {
  match expression {
    Expr::Assignment { .. } | Expr::IndexAssignment { .. } | Expr::Call { .. } => true,
    Expr::Function { .. }
    | Expr::Literal { .. }
    | Expr::Variable { .. }
    | Expr::ModuleAccess { .. } => false,
    Expr::Comment { expression, .. }
    | Expr::Group { expression }
    | Expr::Unary { expression, .. } => has_possible_side_effect(&expression.expr),
    Expr::Index { expression, index } => {
      has_possible_side_effect(&expression.expr) || has_possible_side_effect(&index.expr)
    }
    Expr::Binary { left, right, .. } => {
      has_possible_side_effect(&left.expr) || has_possible_side_effect(&right.expr)
    }
    Expr::List { items } => items.iter().any(|e| has_possible_side_effect(&e.expr)),
    Expr::Dictionary { items } => items.iter().any(|(key, value)| {
      has_possible_side_effect(&key.expr) || has_possible_side_effect(&value.expr)
    }),
    Expr::FormatString { expressions, .. } => expressions
      .iter()
      .any(|e| has_possible_side_effect(&e.expr)),
  }
}

lint_rule! {
  name: NoSideEffectInIndex;
  title: "No Side Effects in Index Assignment";
  message: "Index can be evaluated in an unexpected order, don't have side effects";
  visitor: {
    fn exit_expression(&mut self, expression: &Expression) {
      if let Expr::IndexAssignment { index, .. } = &expression.expr
        && has_possible_side_effect(&index.expr) {
        self.issues.push(index.span);
      }
    }
  }
}
