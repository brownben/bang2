use crate::{
  ast::{Expr, Stmt, Visitor},
  diagnostic::Diagnostic,
  tokens::{LineNumber, Token, TokenType},
  value::Value,
};

trait LintRule<'s> {
  fn check(source: &'s str, ast: &[Stmt]) -> Diagnostic;
}

macro_rules! lint_rule {
  {
    name: $rule_name:ident;
    title: $title:expr;
    message: $message:expr;
    visitor: $visitor:tt
  } => {
    pub struct $rule_name {
      issues: Vec<LineNumber>,
    }
    impl<'s> LintRule<'s> for $rule_name {
      fn check(source: &'s str, ast: &[Stmt]) -> Diagnostic {
        let mut visitor = Self { issues: Vec::new() };
        visitor.visit(ast, source);

        Diagnostic {
          title: $title.to_string(),
          message: $message.to_string(),
          lines: visitor.issues,
        }
      }
    }
    impl <'s> Visitor<&'s str> for $rule_name $visitor
  }
}

lint_rule! {
  name: NoConstantCondition;
  title: "No Constant Conditions";
  message: "The control flow could be removed, as the condition is always true or false";
  visitor: {
    fn exit_statement(&mut self, statement: &Stmt, _source: &str) {
      match statement {
        Stmt::If {
          if_token,
          condition,
          ..
        } => {
          if condition.is_constant() {
            self.issues.push(if_token.line);
          }
        }
        Stmt::While {
          token, condition, ..
        } => {
          if condition.is_constant() {
            self.issues.push(token.line);
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
    fn exit_expression(&mut self, expression: &Expr, _source: &str) {
      if let Expr::Binary { left, right, operator, ..} = expression
        && let TokenType::EqualEqual | TokenType::BangEqual = operator.ttype
        && let Expr::Variable { .. } = right.as_ref()
        && let Expr::Literal { .. } = left.as_ref()
      {
        self.issues.push(operator.line);
      }
    }
  }
}

lint_rule! {
  name: NoNegativeZero;
  title: "No Negative Zero";
  message: "Negative zero is unnecessary as 0 == -0";
  visitor: {
    fn exit_expression(&mut self, expression: &Expr, _source: &str) {
      if let Expr::Unary { expression, .. } = expression
        && let Expr::Literal { value, token, .. } = expression.as_ref()
        && Value::parse_number(value) == Value::from(0.0)
      {
        self.issues.push(token.line);
      }
    }
  }
}

lint_rule! {
  name: NoSelfAssign;
  title: "No Self Assign";
  message: "Assigning a variable to itself is unnecessary";
  visitor: {
    fn exit_expression(&mut self, expression: &Expr, source: &str) {
      if let Expr::Assignment {
        identifier,
        expression,
        ..
      } = expression
      {
        if let Expr::Variable { token, .. } = expression.as_ref()
          && identifier.get_value(source.as_bytes()) == token.get_value(source.as_bytes())
        {
          self.issues.push(identifier.line);
        }
      }
    }
  }
}

lint_rule! {
  name: NoUnreachable;
  title: "No Unreachable Code";
  message: "Code after a return can never be executed";
  visitor:{
    fn exit_statement(&mut self, statement: &Stmt, _source: &str) {
      if let Stmt::Block { body, .. } = statement {
        let mut seen_return: Option<Token> = None;
        for statement in body {
          if let Some(token) = seen_return {
            self.issues.push(token.line);
            break;
          }

          if let Stmt::Return { token, .. } = statement {
            seen_return = Some(**token);
          }
        }
      }
    }
  }
}

pub fn lint(source: &str, ast: &[Stmt]) -> Vec<Diagnostic> {
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
