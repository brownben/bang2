use crate::{
  ast::{BinaryOperator, Expr, Expression, Span, Statement, Stmt, Visitor},
  diagnostic::Diagnostic,
  value::Value,
};

trait LintRule {
  fn check(source: &str, ast: &[Statement]) -> Diagnostic;
}

macro_rules! lint_rule {
  {
    name: $rule_name:ident;
    title: $title:expr;
    message: $message:expr;
    visitor: $visitor:tt
  } => {
    pub struct $rule_name {
      issues: Vec<Span>,
    }
    impl LintRule for $rule_name {
      fn check(source: &str, ast: &[Statement]) -> Diagnostic {
        let mut visitor = Self { issues: Vec::new() };
        visitor.visit(ast);

        Diagnostic {
          title: $title.to_string(),
          message: $message.to_string(),
          lines: visitor.issues.iter().map(|span| span.get_line_number(source)).collect(),
        }
      }
    }
    impl Visitor for $rule_name $visitor
  }
}

lint_rule! {
  name: NoConstantCondition;
  title: "No Constant Conditions";
  message: "The control flow could be removed, as the condition is always true or false";
  visitor: {
    fn exit_statement(&mut self, statement: &Statement) {
      match &statement.stmt {
        Stmt::If { condition, .. } => {
          if condition.is_constant() {
            self.issues.push(statement.span);
          }
        }
        Stmt::While { condition, .. } => {
          if condition.is_constant() {
            self.issues.push(statement.span);
          }
        }
        _ => {}
      }
    }
  }
}

lint_rule! {
  name: NoYodaEquality;
  title: "No Yoda Equality";
  message: "It is clearer to have the variable first then the value to compare to";
  visitor: {
    fn exit_expression(&mut self, expression: &Expression) {
      if let Expr::Binary { left, right, operator, ..} = &expression.expr
        && let BinaryOperator::Equal | BinaryOperator::NotEqual = operator
        && let Expr::Variable { .. } = right.expr
        && let Expr::Literal { .. } = left.expr
      {
        self.issues.push(expression.span);
      }
    }
  }
}

lint_rule! {
  name: NoNegativeZero;
  title: "No Negative Zero";
  message: "Negative zero is unnecessary as 0 == -0";
  visitor: {
    fn exit_expression(&mut self, expression: &Expression) {
      if let Expr::Unary { expression, .. } = &expression.expr
        && let Expr::Literal { value,  .. } = &expression.expr
        && Value::parse_number(value) == Value::from(0.0)
      {
        self.issues.push(expression.span);
      }
    }
  }
}

lint_rule! {
  name: NoSelfAssign;
  title: "No Self Assign";
  message: "Assigning a variable to itself is unnecessary";
  visitor: {
    fn exit_expression(&mut self, expression: &Expression) {
      if let Expr::Assignment {
        identifier,
        expression,
        ..
      } = &expression.expr
      {
        if let Expr::Variable { name, .. } = &expression.expr
          && identifier == name
        {
          self.issues.push(expression.span);
        }
      }
    }
  }
}

lint_rule! {
  name: NoUnreachable;
  title: "No Unreachable Code";
  message: "Code after a return can never be executed";
  visitor: {
    fn exit_statement(&mut self, statement: &Statement) {
      if let Stmt::Block { body, .. } = &statement.stmt {
        let mut seen_return = false;
        for statement in body {
          if seen_return {
            self.issues.push(statement.span);
            break;
          }

          if let Stmt::Return { .. } = statement.stmt {
            seen_return = true;
          }
        }
      }
    }
  }
}

pub fn lint(source: &str, ast: &[Statement]) -> Vec<Diagnostic> {
  let mut results = vec![
    NoConstantCondition::check(source, ast),
    NoYodaEquality::check(source, ast),
    NoNegativeZero::check(source, ast),
    NoSelfAssign::check(source, ast),
    NoUnreachable::check(source, ast),
  ];

  results.retain(|r| !r.lines.is_empty());
  results
}
