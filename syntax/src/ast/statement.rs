use super::expression::Expression;
use super::types::TypeExpression;
use super::Span;
use std::fmt;

#[derive(Clone, Debug)]
pub struct Statement<'s> {
  pub stmt: Stmt<'s>,
  pub span: Span,
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

#[derive(Clone, Debug)]
pub enum Stmt<'source> {
  Block {
    body: Vec<Statement<'source>>,
  },
  Declaration {
    identifier: DeclarationIdentifier<'source>,
    type_: Option<TypeExpression<'source>>,
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
    items: Vec<AliasItem<'source>>,
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
pub struct AliasItem<'s> {
  pub name: &'s str,
  pub span: Span,
  pub alias: Option<&'s str>,
}
impl<'s> AliasItem<'s> {
  pub fn get_name(&self) -> &'s str {
    self.alias.unwrap_or(self.name)
  }
}

#[derive(Clone, Debug)]
pub enum DeclarationIdentifier<'source> {
  Variable(&'source str),
  Ordered(Vec<&'source str>),
  Named(Vec<AliasItem<'source>>),
}
impl fmt::Display for DeclarationIdentifier<'_> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      DeclarationIdentifier::Variable(identifier) => write!(f, "{identifier}"),
      DeclarationIdentifier::Ordered(identifiers) => {
        write!(f, "{}", identifiers.join(", "))
      }
      DeclarationIdentifier::Named(identifiers) => {
        for identifier in identifiers {
          write!(f, "{}", identifier.name)?;
          if let Some(alias) = identifier.alias {
            write!(f, " as {alias}")?;
          }
        }
        write!(f, ", ")
      }
    }
  }
}
