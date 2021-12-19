use crate::token::Token;

use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum LiteralValue {
  String(Rc<str>),
  Number(f64),
  True,
  False,
  Null,
}

impl std::fmt::Display for LiteralValue {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      Self::True => write!(f, "true"),
      Self::False => write!(f, "false"),
      Self::Null => write!(f, "null"),
      Self::Number(value) => write!(f, "{}", value),
      Self::String(value) => write!(f, "{}", value),
    }
  }
}

#[derive(Debug, Clone)]
pub enum BinaryOperator {
  Plus,
  Minus,
  Star,
  Slash,
  BangEqual,
  EqualEqual,
  Greater,
  GreaterEqual,
  Less,
  LessEqual,
  And,
  Or,
  QuestionQuestion,
}

impl std::fmt::Display for BinaryOperator {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      BinaryOperator::Plus => write!(f, "+"),
      BinaryOperator::Minus => write!(f, "-"),
      BinaryOperator::Star => write!(f, "*"),
      BinaryOperator::Slash => write!(f, "/"),
      BinaryOperator::BangEqual => write!(f, "!="),
      BinaryOperator::EqualEqual => write!(f, "=="),
      BinaryOperator::Greater => write!(f, ">"),
      BinaryOperator::GreaterEqual => write!(f, ">="),
      BinaryOperator::Less => write!(f, "<"),
      BinaryOperator::LessEqual => write!(f, "<="),
      BinaryOperator::And => write!(f, "and"),
      BinaryOperator::Or => write!(f, "or"),
      BinaryOperator::QuestionQuestion => write!(f, "??"),
    }
  }
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
  Minus,
  Bang,
}

impl std::fmt::Display for UnaryOperator {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      UnaryOperator::Minus => write!(f, "-"),
      UnaryOperator::Bang => write!(f, "!"),
    }
  }
}

#[derive(Debug, Clone)]
pub struct Parameter {
  pub identifier: Token,
  pub value: String,
  pub type_: String,
}

#[derive(Debug, Clone)]
pub enum Expression {
  Literal {
    value: LiteralValue,
    token: Token,
  },
  Group {
    expression: Box<Expression>,
  },
  Unary {
    token: Token,
    operator: UnaryOperator,
    expression: Box<Expression>,
  },
  Binary {
    token: Token,
    left: Box<Expression>,
    operator: BinaryOperator,
    right: Box<Expression>,
  },
  Assignment {
    identifier: Token,
    variable_name: String,
    expression: Box<Expression>,
  },
  Variable {
    variable_name: String,
    identifier: Token,
  },
  Call {
    expression: Box<Expression>,
    token: Token,
    arguments: Vec<Expression>,
  },
  Function {
    token: Token,
    parameters: Vec<Parameter>,
    body: Box<Statement>,
    return_type: Option<String>,
  },
}

impl Expression {
  pub fn has_side_effect(&self) -> bool {
    match self {
      Expression::Variable { .. } => false,
      Expression::Literal { .. } => false,
      Expression::Assignment { .. } => true,
      Expression::Group { expression, .. } => expression.has_side_effect(),
      Expression::Unary { expression, .. } => expression.has_side_effect(),
      Expression::Binary { left, right, .. } => left.has_side_effect() || right.has_side_effect(),
      Expression::Call { .. } => true,
      Expression::Function { .. } => false,
    }
  }
}

#[derive(Debug, Clone)]
pub enum Statement {
  Declaration {
    token: Token,
    identifier: Token,
    variable_name: String,
    expression: Option<Expression>,
  },
  If {
    if_token: Token,
    else_token: Option<Token>,
    condition: Expression,
    then: Box<Statement>,
    otherwise: Option<Box<Statement>>,
  },
  While {
    token: Token,
    condition: Expression,
    body: Box<Statement>,
  },
  Block {
    body: Vec<Statement>,
  },
  Expression {
    expression: Expression,
  },
  Return {
    token: Token,
    expression: Option<Expression>,
  },
}

pub trait Visitor {
  fn visit(&mut self, statements: &[Statement]) {
    statements.iter().for_each(|s| self.visit_statement(s));
  }

  fn visit_statement(&mut self, statement: &Statement) {
    match statement {
      Statement::Block { body, .. } => body.iter().for_each(|s| self.visit_statement(s)),
      Statement::Declaration { expression, .. } => {
        if let Some(expression) = &*expression {
          self.visit_expression(expression)
        }
      }
      Statement::Expression { expression, .. } => self.visit_expression(expression),
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

    self.exit_statement(statement)
  }

  fn visit_expression(&mut self, expression: &Expression) {
    match expression {
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
      Expression::Function { body, .. } => self.visit_statement(body),
      Expression::Group { expression, .. } => self.visit_expression(expression),
      Expression::Literal { .. } => {}
      Expression::Unary { expression, .. } => self.visit_expression(expression),
      Expression::Variable { .. } => {}
    }

    self.exit_expression(expression)
  }

  fn exit_expression(&mut self, _expression: &Expression) {}
  fn exit_statement(&mut self, _statement: &Statement) {}
}
