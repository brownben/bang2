use super::statement::Statement;
use super::types::TypeExpression;
use super::Span;
use crate::tokens::TokenType;
use std::{fmt, ops};

#[derive(Clone, Debug)]
pub struct Expression<'s> {
  pub expr: Expr<'s>,
  pub span: Span,
}
impl<'s> ops::Deref for Expression<'s> {
  type Target = Expr<'s>;
  fn deref(&self) -> &Expr<'s> {
    &self.expr
  }
}

macro_rules! expression {
    ($type:ident $struct:tt, ($start:expr, $end:expr)) => {{
      let start = $start;
      let end = $end;

      Expression {
        expr: Expr::$type $struct,
        span: Span { start: start.start, end: end.end  },
      }
    }};

    ($type:ident $struct:tt, $range:expr) => {
      expression!($type $struct, ($range, $range))
    };
  }
pub(crate) use expression;

#[derive(Clone, Debug)]
pub enum Expr<'source> {
  Assignment {
    identifier: &'source str,
    expression: Box<Expression<'source>>,
  },
  Binary {
    operator: BinaryOperator,
    left: Box<Expression<'source>>,
    right: Box<Expression<'source>>,
  },
  Call {
    expression: Box<Expression<'source>>,
    arguments: Vec<Expression<'source>>,
  },
  Comment {
    expression: Box<Expression<'source>>,
    text: &'source str,
  },
  FormatString {
    strings: Vec<String>,
    expressions: Vec<Expression<'source>>,
  },
  Function {
    parameters: Vec<Parameter<'source>>,
    return_type: Option<TypeExpression<'source>>,
    body: Box<Statement<'source>>,
    name: Option<&'source str>,
  },
  Group {
    expression: Box<Expression<'source>>,
  },
  Index {
    expression: Box<Expression<'source>>,
    index: Box<Expression<'source>>,
  },
  IndexAssignment {
    expression: Box<Expression<'source>>,
    index: Box<Expression<'source>>,
    value: Box<Expression<'source>>,
    assignment_operator: Option<AssignmentOperator>,
  },
  List {
    items: Vec<Expression<'source>>,
  },
  Literal {
    type_: LiteralType,
    value: &'source str,
  },
  Unary {
    operator: UnaryOperator,
    expression: Box<Expression<'source>>,
  },
  Variable {
    name: &'source str,
  },
}
impl<'s> Expr<'s> {
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
      } => left.is_constant() && right.is_constant() && *operator != BinaryOperator::Pipeline,
      Expr::List { items } => items.iter().all(|item| item.is_constant()),
      Expr::Index {
        expression, index, ..
      } => expression.is_constant() && index.is_constant(),
      Expr::IndexAssignment { value, .. } => value.is_constant(),
      Expr::FormatString { expressions, .. } => expressions.iter().all(|e| e.is_constant()),
    }
  }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LiteralType {
  String,
  Number,
  True,
  False,
  Null,
}
impl From<TokenType> for LiteralType {
  fn from(token_type: TokenType) -> Self {
    match token_type {
      TokenType::String => Self::String,
      TokenType::Number => Self::Number,
      TokenType::True => Self::True,
      TokenType::False => Self::False,
      TokenType::Null => Self::Null,
      _ => unreachable!(),
    }
  }
}
impl fmt::Display for LiteralType {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match self {
      Self::String => write!(f, "string"),
      Self::Number => write!(f, "number"),
      Self::True => write!(f, "true"),
      Self::False => write!(f, "false"),
      Self::Null => write!(f, "null"),
    }
  }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BinaryOperator {
  Plus,
  Minus,
  Multiply,
  Divide,
  NotEqual,
  Equal,
  Greater,
  GreaterEqual,
  Less,
  LessEqual,
  And,
  Or,
  Nullish,
  Pipeline,
}
impl From<TokenType> for BinaryOperator {
  fn from(token_type: TokenType) -> Self {
    match token_type {
      TokenType::Plus => Self::Plus,
      TokenType::Minus => Self::Minus,
      TokenType::Star => Self::Multiply,
      TokenType::Slash => Self::Divide,
      TokenType::BangEqual => Self::NotEqual,
      TokenType::EqualEqual => Self::Equal,
      TokenType::Greater => Self::Greater,
      TokenType::GreaterEqual => Self::GreaterEqual,
      TokenType::Less => Self::Less,
      TokenType::LessEqual => Self::LessEqual,
      TokenType::And => Self::And,
      TokenType::Or => Self::Or,
      TokenType::QuestionQuestion => Self::Nullish,
      TokenType::RightRight => Self::Pipeline,
      _ => unreachable!(),
    }
  }
}
impl fmt::Display for BinaryOperator {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Self::Plus => write!(f, "+"),
      Self::Minus => write!(f, "-"),
      Self::Multiply => write!(f, "*"),
      Self::Divide => write!(f, "/"),
      Self::NotEqual => write!(f, "!="),
      Self::Equal => write!(f, "=="),
      Self::Greater => write!(f, ">"),
      Self::GreaterEqual => write!(f, ">="),
      Self::Less => write!(f, "<"),
      Self::LessEqual => write!(f, "<="),
      Self::And => write!(f, "and"),
      Self::Or => write!(f, "or"),
      Self::Nullish => write!(f, "??"),
      Self::Pipeline => write!(f, ">>"),
    }
  }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AssignmentOperator {
  Plus,
  Minus,
  Multiply,
  Divide,
}
impl AssignmentOperator {
  pub fn from_binary(operator: &BinaryOperator) -> Option<Self> {
    match operator {
      BinaryOperator::Plus => Some(Self::Plus),
      BinaryOperator::Minus => Some(Self::Minus),
      BinaryOperator::Multiply => Some(Self::Multiply),
      BinaryOperator::Divide => Some(Self::Divide),
      _ => None,
    }
  }

  pub fn from_token(operator: TokenType) -> Option<Self> {
    match operator {
      TokenType::PlusEqual => Some(Self::Plus),
      TokenType::MinusEqual => Some(Self::Minus),
      TokenType::StarEqual => Some(Self::Multiply),
      TokenType::SlashEqual => Some(Self::Divide),
      _ => None,
    }
  }

  pub fn token_to_binary(token_type: TokenType) -> BinaryOperator {
    match token_type {
      TokenType::PlusEqual => BinaryOperator::Plus,
      TokenType::MinusEqual => BinaryOperator::Minus,
      TokenType::StarEqual => BinaryOperator::Multiply,
      TokenType::SlashEqual => BinaryOperator::Divide,
      _ => unreachable!("The only supported assignment operators are: +, -, *, /"),
    }
  }
}
impl fmt::Display for AssignmentOperator {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Self::Plus => write!(f, "+="),
      Self::Minus => write!(f, "-="),
      Self::Multiply => write!(f, "*="),
      Self::Divide => write!(f, "/="),
    }
  }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum UnaryOperator {
  Not,
  Minus,
}
impl From<TokenType> for UnaryOperator {
  fn from(token_type: TokenType) -> Self {
    match token_type {
      TokenType::Bang => Self::Not,
      TokenType::Minus => Self::Minus,
      _ => unreachable!(),
    }
  }
}
impl fmt::Display for UnaryOperator {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Self::Not => write!(f, "!"),
      Self::Minus => write!(f, "-"),
    }
  }
}

#[derive(Clone, Debug)]
pub struct Parameter<'s> {
  pub name: &'s str,
  pub span: Span,
  pub type_: Option<TypeExpression<'s>>,
  pub catch_remaining: bool,
}
