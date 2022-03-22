use crate::tokens::{Token, TokenType};

type TokenRef<'source> = &'source Token;

pub trait GetPosition<'s> {
  fn get_start(&'s self) -> TokenRef<'s>;
  fn get_end(&'s self) -> TokenRef<'s>;
}

#[derive(Debug)]
pub enum Expr<'source> {
  Assignment {
    identifier: TokenRef<'source>,
    expression: Box<Expr<'source>>,
  },
  Binary {
    operator: TokenRef<'source>,
    left: Box<Expr<'source>>,
    right: Box<Expr<'source>>,
  },
  Call {
    token: TokenRef<'source>,
    end_token: TokenRef<'source>,
    expression: Box<Expr<'source>>,
    arguments: Vec<Expr<'source>>,
  },
  Comment {
    token: TokenRef<'source>,
    expression: Box<Expr<'source>>,
  },
  Function {
    token: TokenRef<'source>,
    parameters: Vec<TokenRef<'source>>,
    body: Box<Stmt<'source>>,
    name: Option<&'source str>,
  },
  Group {
    token: TokenRef<'source>,
    end_token: TokenRef<'source>,
    expression: Box<Expr<'source>>,
  },
  Literal {
    token: TokenRef<'source>,
    value: &'source str,
  },
  Unary {
    operator: TokenRef<'source>,
    expression: Box<Expr<'source>>,
  },
  Variable {
    token: TokenRef<'source>,
  },
}
impl<'s> Expr<'s> {
  pub fn has_side_effect(&self) -> bool {
    match self {
      Expr::Assignment { .. } | Expr::Call { .. } => true,
      Expr::Function { .. } | Expr::Variable { .. } | Expr::Literal { .. } => false,
      Expr::Group { expression, .. }
      | Expr::Unary { expression, .. }
      | Expr::Comment { expression, .. } => expression.has_side_effect(),
      Expr::Binary {
        left,
        right,
        operator,
        ..
      } => {
        left.has_side_effect() || right.has_side_effect() || operator.ttype == TokenType::RightRight
      }
    }
  }

  pub fn is_constant(&self) -> bool {
    match self {
      Expr::Call { .. } | Expr::Variable { .. } => false,
      Expr::Function { .. } | Expr::Literal { .. } => true,
      Expr::Group { expression, .. }
      | Expr::Unary { expression, .. }
      | Expr::Assignment { expression, .. }
      | Expr::Comment { expression, .. } => expression.is_constant(),
      Expr::Binary {
        left,
        right,
        operator,
        ..
      } => left.is_constant() && right.is_constant() && operator.ttype != TokenType::RightRight,
    }
  }
}
impl<'s> GetPosition<'s> for Expr<'s> {
  fn get_start(&'s self) -> TokenRef<'s> {
    match self {
      Expr::Call { token, .. }
      | Expr::Comment { token, .. }
      | Expr::Function { token, .. }
      | Expr::Group { token, .. }
      | Expr::Literal { token, .. }
      | Expr::Variable { token, .. } => token,
      Expr::Unary { operator, .. } => operator,
      Expr::Assignment { identifier, .. } => identifier,
      Expr::Binary { left, .. } => left.get_start(),
    }
  }

  fn get_end(&'s self) -> TokenRef<'s> {
    match self {
      Expr::Comment { token, .. } | Expr::Literal { token, .. } | Expr::Variable { token, .. } => {
        token
      }
      Expr::Call { end_token, .. } | Expr::Group { end_token, .. } => end_token,
      Expr::Assignment { identifier, .. } => identifier,
      Expr::Binary { right, .. } => right.get_end(),
      Expr::Function { body, .. } => body.get_end(),
      Expr::Unary { expression, .. } => expression.get_end(),
    }
  }
}

#[derive(Debug)]
pub enum Stmt<'source> {
  Block {
    body: Vec<Stmt<'source>>,
  },
  Declaration {
    token: TokenRef<'source>,
    identifier: TokenRef<'source>,
    expression: Option<Expr<'source>>,
  },
  Expression {
    expression: Expr<'source>,
  },
  If {
    if_token: TokenRef<'source>,
    else_token: Option<TokenRef<'source>>,
    condition: Expr<'source>,
    then: Box<Stmt<'source>>,
    otherwise: Option<Box<Stmt<'source>>>,
  },
  Import {
    token: TokenRef<'source>,
    module: TokenRef<'source>,
    items: Vec<TokenRef<'source>>,
    end_token: TokenRef<'source>,
  },
  Return {
    token: TokenRef<'source>,
    expression: Option<Expr<'source>>,
  },
  While {
    token: TokenRef<'source>,
    condition: Expr<'source>,
    body: Box<Stmt<'source>>,
  },
  Comment {
    token: TokenRef<'source>,
  },
}
impl<'s> GetPosition<'s> for Stmt<'s> {
  fn get_start(&'s self) -> TokenRef<'s> {
    match self {
      Stmt::Block { body, .. } => body.first().unwrap().get_start(),
      Stmt::Expression { expression, .. } => expression.get_start(),
      Stmt::If { if_token, .. } => if_token,
      Stmt::Comment { token, .. }
      | Stmt::Declaration { token, .. }
      | Stmt::Import { token, .. }
      | Stmt::Return { token, .. }
      | Stmt::While { token, .. } => token,
    }
  }

  fn get_end(&'s self) -> TokenRef<'s> {
    match self {
      Stmt::Block { body, .. } => body.last().unwrap().get_end(),
      Stmt::Declaration {
        identifier,
        expression,
        ..
      } => {
        if let Some(expression) = expression {
          expression.get_end()
        } else {
          identifier
        }
      }
      Stmt::Expression { expression, .. } => expression.get_end(),
      Stmt::If {
        then, otherwise, ..
      } => {
        if let Some(otherwise) = otherwise {
          otherwise.get_end()
        } else {
          then.get_end()
        }
      }
      Stmt::Import { end_token, .. } => end_token,
      Stmt::Return {
        token, expression, ..
      } => {
        if let Some(expression) = expression {
          expression.get_end()
        } else {
          token
        }
      }
      Stmt::While { body, .. } => body.get_end(),
      Stmt::Comment { token, .. } => token,
    }
  }
}

pub trait Visitor<T: Copy> {
  fn visit(&mut self, statements: &[Stmt], value: T) {
    statements
      .iter()
      .for_each(|s| self.visit_statement(s, value));
  }

  fn visit_statement(&mut self, statement: &Stmt, value: T) {
    match statement {
      Stmt::Block { body, .. } => body.iter().for_each(|s| self.visit_statement(s, value)),
      Stmt::Declaration { expression, .. } => {
        if let Some(expression) = &*expression {
          self.visit_expression(expression, value);
        }
      }
      Stmt::Expression { expression, .. } => self.visit_expression(expression, value),
      Stmt::If {
        condition,
        then,
        otherwise,
        ..
      } => {
        self.visit_expression(condition, value);
        self.visit_statement(then, value);
        if let Some(otherwise) = &*otherwise {
          self.visit_statement(otherwise.as_ref(), value);
        }
      }
      Stmt::Return { expression, .. } => {
        if let Some(expression) = expression {
          self.visit_expression(expression, value);
        }
      }
      Stmt::While {
        condition, body, ..
      } => {
        self.visit_expression(condition, value);
        self.visit_statement(body, value);
      }
      Stmt::Import { .. } | Stmt::Comment { .. } => {}
    }

    self.exit_statement(statement, value);
  }

  fn visit_expression(&mut self, expression: &Expr, value: T) {
    match expression {
      Expr::Assignment { expression, .. }
      | Expr::Comment { expression, .. }
      | Expr::Group { expression, .. }
      | Expr::Unary { expression, .. } => self.visit_expression(expression, value),
      Expr::Binary { left, right, .. } => {
        self.visit_expression(left, value);
        self.visit_expression(right, value);
      }
      Expr::Call {
        expression,
        arguments,
        ..
      } => {
        self.visit_expression(expression, value);
        arguments
          .iter()
          .for_each(|arg| self.visit_expression(arg, value));
      }
      Expr::Function { body, .. } => self.visit_statement(body, value),
      Expr::Literal { .. } | Expr::Variable { .. } => {}
    }

    self.exit_expression(expression, value);
  }

  fn exit_expression(&mut self, _expression: &Expr, _value: T) {}
  fn exit_statement(&mut self, _statement: &Stmt, _value: T) {}
}
