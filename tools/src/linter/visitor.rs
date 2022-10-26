use bang_syntax::ast::{expression, statement};

pub trait Visitor {
  fn visit(&mut self, statements: &[statement::Statement]) {
    statements.iter().for_each(|s| self.visit_statement(s));
    self.exit_ast();
  }

  fn visit_statement(&mut self, statement: &statement::Statement) {
    use statement::Stmt;
    self.enter_statement(statement);

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
    self.enter_expression(expression);

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
      Expr::Literal { .. } | Expr::Variable { .. } | Expr::ModuleAccess { .. } => {}
    }

    self.exit_expression(expression);
  }

  fn enter_expression(&mut self, _expression: &expression::Expression) {}
  fn enter_statement(&mut self, _statement: &statement::Statement) {}

  fn exit_expression(&mut self, _expression: &expression::Expression) {}
  fn exit_statement(&mut self, _statement: &statement::Statement) {}

  fn exit_ast(&mut self) {}
}
