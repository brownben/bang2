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

#[derive(Debug, Clone)]
pub enum UnaryOperator {
  Minus,
  Bang,
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
pub struct Parameter {
  pub identifier: Token,
  pub value: String,
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
  Function {
    name: String,
    token: Token,
    identifier: Token,
    parameters: Vec<Parameter>,
    body: Box<Statement>,
  },
  Return {
    token: Token,
    expression: Option<Expression>,
  },
}
