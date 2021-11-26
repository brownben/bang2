use crate::scanner::Token;
use crate::value::Value;

pub enum Expression {
  Literal{value:Value, token:Token},
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
    expression: Box<Expression>,
    global: bool,
  },
  Variable {
    identifier: Token,
    global: bool,
  },
}

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
  Print {
    token: Token,
    expression: Expression,
  },
  Block {
    body: Vec<Statement>,
  },
  Expression {
    expression: Expression,
  },
}
