use crate::ast::{Expression, LiteralValue, Statement, Visitor};
use crate::token::Token;

pub struct RuleResult {
  pub name: &'static str,
  pub message: &'static str,
  pub issues: Vec<Token>,
}

pub trait Rule {
  fn check(ast: &[Statement]) -> Option<RuleResult>;
}

pub struct NoNegativeZero {
  issues: Vec<Token>,
}

impl Rule for NoNegativeZero {
  fn check(ast: &[Statement]) -> Option<RuleResult> {
    let mut visitor = NoNegativeZero { issues: Vec::new() };
    visitor.visit(ast);

    if visitor.issues.is_empty() {
      None
    } else {
      Some(RuleResult {
        name: "No Negative Zero",
        message: "Negative zero is unnecessary as 0 == -0",
        issues: visitor.issues,
      })
    }
  }
}

impl Visitor for NoNegativeZero {
  fn visit_expression(&mut self, expression: &Expression) {
    match expression {
      Expression::Unary {
        expression,
        operator,
        ..
      } => {
        if let Expression::Literal { value, .. } = expression.as_ref() {
          if let LiteralValue::Number(num) = value {
            if *num == 0.0 {
              self.issues.push(*operator);
            }
          }
        } else {
          self.visit_expression(expression)
        }
      }
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
      Expression::Variable { .. } => {}
    }
  }
}

pub struct NoUnreachable {
  issues: Vec<Token>,
}

impl Rule for NoUnreachable {
  fn check(ast: &[Statement]) -> Option<RuleResult> {
    let mut visitor = NoUnreachable { issues: Vec::new() };

    visitor.visit(ast);

    if visitor.issues.is_empty() {
      None
    } else {
      Some(RuleResult {
        name: "No Unreachable Code",
        message: "Code after a return can never be executed",
        issues: visitor.issues,
      })
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
