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
    operator: Token,
    expression: Box<Expression>,
  },
  Binary {
    left: Box<Expression>,
    operator: Token,
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
