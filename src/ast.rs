use std::ops::{Deref, Range};

use crate::tokens::{CharacterPosition, LineNumber, Token, TokenType};

#[derive(Copy, Clone, Debug, PartialEq)]
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
  PlusEqual,
  MinusEqual,
  MultiplyEqual,
  DivideEqual,
}
impl BinaryOperator {
  pub fn get_corresponding_assignment_operator(&self) -> Option<Self> {
    match self {
      Self::PlusEqual | Self::Plus => Some(Self::PlusEqual),
      Self::MinusEqual | Self::Minus => Some(Self::MinusEqual),
      Self::MultiplyEqual | Self::Multiply => Some(Self::MultiplyEqual),
      Self::DivideEqual | Self::Divide => Some(Self::DivideEqual),
      _ => None,
    }
  }
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
      TokenType::PlusEqual => Self::PlusEqual,
      TokenType::MinusEqual => Self::MinusEqual,
      TokenType::StarEqual => Self::MultiplyEqual,
      TokenType::SlashEqual => Self::DivideEqual,
      _ => unreachable!(),
    }
  }
}
impl std::fmt::Display for BinaryOperator {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
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
      Self::PlusEqual => write!(f, "+="),
      Self::MinusEqual => write!(f, "-="),
      Self::MultiplyEqual => write!(f, "*="),
      Self::DivideEqual => write!(f, "/="),
    }
  }
}

#[derive(Copy, Clone, Debug, PartialEq)]
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
impl std::fmt::Display for UnaryOperator {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      Self::Not => write!(f, "!"),
      Self::Minus => write!(f, "-"),
    }
  }
}

#[derive(Copy, Clone, Debug, PartialEq)]
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

#[derive(Copy, Clone, Debug)]
pub struct ImportItem<'s> {
  pub name: &'s str,
  pub span: Span,
}

#[derive(Copy, Clone, Debug)]
pub struct Parameter<'s> {
  pub name: &'s str,
  pub span: Span,
}

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
  Function {
    parameters: Vec<Parameter<'source>>,
    body: Box<Statement<'source>>,
    name: Option<&'source str>,
  },
  Group {
    expression: Box<Expression<'source>>,
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
        left.has_side_effect() || right.has_side_effect() || *operator == BinaryOperator::Pipeline
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
      } => left.is_constant() && right.is_constant() && *operator != BinaryOperator::Pipeline,
    }
  }
}

#[derive(Clone, Debug)]
pub enum Stmt<'source> {
  Block {
    body: Vec<Statement<'source>>,
  },
  Declaration {
    identifier: &'source str,
    expression: Option<Expression<'source>>,
  },
  Expression {
    expression: Expression<'source>,
  },
  If {
    condition: Expression<'source>,
    then: Box<Statement<'source>>,
    otherwise: Option<Box<Statement<'source>>>,
  },
  Import {
    module: &'source str,
    items: Vec<ImportItem<'source>>,
  },
  Return {
    expression: Option<Expression<'source>>,
  },
  While {
    condition: Expression<'source>,
    body: Box<Statement<'source>>,
  },
  Comment {
    text: &'source str,
  },
}

#[derive(Copy, Clone, Debug)]
pub struct Span {
  pub start: CharacterPosition,
  pub end: CharacterPosition,
}
impl Span {
  pub fn get_line_number(&self, source: &str) -> LineNumber {
    let mut line = 1;

    for (i, byte) in source.as_bytes().iter().enumerate() {
      if *byte == b'\n' {
        line += 1;
      }

      if i == self.start as usize {
        return line as LineNumber;
      }
    }

    unreachable!()
  }

  pub fn get_line_number_end(&self, source: &str) -> LineNumber {
    let mut line = 1;

    for (i, byte) in source.as_bytes().iter().enumerate() {
      if *byte == b'\n' {
        line += 1;
      }

      if i == self.end as usize {
        return line as LineNumber;
      }
    }

    unreachable!()
  }
}
impl From<Range<CharacterPosition>> for Span {
  fn from(range: Range<CharacterPosition>) -> Self {
    Span {
      start: range.start,
      end: range.end,
    }
  }
}
impl From<&Token> for Span {
  fn from(token: &Token) -> Self {
    Span {
      start: token.start,
      end: token.end,
    }
  }
}

#[derive(Clone, Debug)]
pub struct Expression<'s> {
  pub expr: Expr<'s>,
  pub span: Span,
}
impl<'s> Deref for Expression<'s> {
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
      span: Span { start: start.start, end: end.end  }
    }
  }};

  ($type:ident $struct:tt, $range:expr) => {
    expression!($type $struct, ($range, $range))
  };
}
pub(crate) use expression;

#[derive(Clone, Debug)]
pub struct Statement<'s> {
  pub stmt: Stmt<'s>,
  pub span: Span,
}
impl<'s> Deref for Statement<'s> {
  type Target = Stmt<'s>;
  fn deref(&self) -> &Stmt<'s> {
    &self.stmt
  }
}

macro_rules! statement {
  ($type:ident $struct:tt, ($start:expr, $end:expr)) => {{
    let start = $start;
    let end = $end;

    Statement {
      stmt: Stmt::$type $struct,
      span: Span { start: start.start, end: end.end }
    }
  }};

  ($type:ident $struct:tt, $range:expr) => {
    statement!($type $struct, ($range, $range))
  };
}
pub(crate) use statement;

pub trait Visitor {
  fn visit(&mut self, statements: &[Statement]) {
    statements.iter().for_each(|s| self.visit_statement(s));
  }

  fn visit_statement(&mut self, statement: &Statement) {
    match &statement.stmt {
      Stmt::Block { body, .. } => body.iter().for_each(|s| self.visit_statement(s)),
      Stmt::Declaration { expression, .. } => {
        if let Some(expression) = &*expression {
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
        if let Some(otherwise) = &*otherwise {
          self.visit_statement(otherwise.as_ref());
        }
      }
      Stmt::Return { expression, .. } => {
        if let Some(expression) = expression {
          self.visit_expression(expression);
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

  fn visit_expression(&mut self, expression: &Expression) {
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
      Expr::Function { body, .. } => self.visit_statement(body),
      Expr::Literal { .. } | Expr::Variable { .. } => {}
    }

    self.exit_expression(expression);
  }

  fn exit_expression(&mut self, _expression: &Expression) {}
  fn exit_statement(&mut self, _statement: &Statement) {}
}
