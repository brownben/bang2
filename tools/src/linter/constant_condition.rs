use super::{lint_rule, Diagnostic, LintRule, Visitor};
use bang_syntax::ast::{
  expression::{operators, Expr},
  statement::{Statement, Stmt},
  Span,
};

pub fn is_constant(expr: &Expr) -> bool {
  match expr {
    Expr::Call { .. } | Expr::Variable { .. } => false,
    Expr::Function { .. } | Expr::Literal { .. } | Expr::ModuleAccess { .. } => true,
    Expr::Group { expression, .. }
    | Expr::Unary { expression, .. }
    | Expr::Assignment { expression, .. }
    | Expr::Comment { expression, .. } => is_constant(&expression.expr),
    Expr::Binary {
      left,
      right,
      operator,
    } => {
      is_constant(&left.expr)
        && is_constant(&right.expr)
        && *operator != operators::Binary::Pipeline
    }
    Expr::List { items } => items.iter().all(|e| is_constant(&e.expr)),
    Expr::Index {
      expression, index, ..
    } => is_constant(&expression.expr) && is_constant(&index.expr),
    Expr::IndexAssignment { value, .. } => is_constant(&value.expr),
    Expr::FormatString { expressions, .. } => expressions.iter().all(|e| is_constant(&e.expr)),
  }
}

lint_rule! {
  name: NoConstantCondition;
  title: "No Constant Conditions";
  message: "The control flow could be removed, as the condition is always true or false";
  visitor: {
    fn exit_statement(&mut self, statement: &Statement) {
      match &statement.stmt {
        Stmt::If { condition, .. } | Stmt::While { condition, .. } => {
          if is_constant(&condition.expr) {
            self.issues.push(statement.span);
          }
        }
        _ => {}
      }
    }
  }
}
