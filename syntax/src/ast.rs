use crate::tokens::{CharacterPosition, LineNumber, Token};
use std::fmt;

pub mod expression;
pub mod statement;
pub mod types;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Span {
  pub start: CharacterPosition,
  pub end: CharacterPosition,
}
impl Span {
  pub fn get_line_number(&self, source: &str) -> LineNumber {
    let mut line: LineNumber = 1;

    for (i, byte) in source.as_bytes().iter().enumerate() {
      if *byte == b'\n' {
        line += 1;
      }

      if i == self.start as usize {
        return line;
      }
    }

    line
  }

  pub fn get_line_number_end(&self, source: &str) -> LineNumber {
    let mut line: LineNumber = 1;

    for (i, byte) in source.as_bytes().iter().enumerate() {
      if *byte == b'\n' {
        line += 1;
      }

      if i == self.end as usize {
        return line;
      }
    }

    line
  }
}
impl From<&Token> for Span {
  fn from(token: &Token) -> Self {
    Self {
      start: token.start,
      end: token.end,
    }
  }
}
impl fmt::Display for Span {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}..{}", self.start, self.end)
  }
}

pub trait Visitor {
  fn visit(&mut self, statements: &[statement::Statement]) {
    statements.iter().for_each(|s| self.visit_statement(s));
  }

  fn visit_statement(&mut self, statement: &statement::Statement) {
    use statement::Stmt;

    match &statement.stmt {
      Stmt::Block { body, .. } => body.iter().for_each(|s| self.visit_statement(s)),
      Stmt::Declaration { expression, .. } | Stmt::Return { expression, .. } => {
        if let Some(expression) = expression {
          self.visit_expression(expression);
        }
      }
      Stmt::Expression { expression, .. } => self.visit_expression(expression),
      Stmt::If {
        condition,
        then,
        otherwise,
        ..
      } => {
        self.visit_expression(condition);
        self.visit_statement(then);
        if let Some(otherwise) = otherwise {
          self.visit_statement(otherwise.as_ref());
        }
      }
      Stmt::While {
        condition, body, ..
      } => {
        self.visit_expression(condition);
        self.visit_statement(body);
      }
      Stmt::Import { .. } | Stmt::Comment { .. } => {}
    }

    self.exit_statement(statement);
  }

  fn visit_expression(&mut self, expression: &expression::Expression) {
    use expression::Expr;

    match &expression.expr {
      Expr::Assignment { expression, .. }
      | Expr::Comment { expression, .. }
      | Expr::Group { expression, .. }
      | Expr::Unary { expression, .. } => self.visit_expression(expression),
      Expr::Binary { left, right, .. } => {
        self.visit_expression(left);
        self.visit_expression(right);
      }
      Expr::Call {
        expression,
        arguments,
        ..
      } => {
        self.visit_expression(expression);
        arguments.iter().for_each(|arg| self.visit_expression(arg));
      }
      Expr::FormatString {
        strings: _,
        expressions,
      } => expressions.iter().for_each(|e| self.visit_expression(e)),
      Expr::Function { body, .. } => self.visit_statement(body),
      Expr::Literal { .. } | Expr::Variable { .. } => {}
      Expr::List { items } => items.iter().for_each(|item| self.visit_expression(item)),
      Expr::Index { expression, index } => {
        self.visit_expression(expression);
        self.visit_expression(index);
      }
      Expr::IndexAssignment {
        expression,
        index,
        value,
        ..
      } => {
        self.visit_expression(expression);
        self.visit_expression(index);
        self.visit_expression(value);
      }
    }

    self.exit_expression(expression);
  }

  fn exit_expression(&mut self, _expression: &expression::Expression) {}
  fn exit_statement(&mut self, _statement: &statement::Statement) {}
}
