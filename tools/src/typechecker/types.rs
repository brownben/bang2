use super::Context;
pub use bang_syntax::ast::expression::LiteralType as Literal;
use std::{fmt::Display, string::ToString};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Truthiness {
  True,
  False,
  Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Function {
  pub parameters: Vec<Type>,
  pub return_type: Box<Type>,
  pub catch_all: bool,
}

pub type Existential = u16;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
  Any,
  Never,
  Literal(Literal),
  List(Box<Type>),
  Set(Box<Type>),
  Dict(Box<Type>, Box<Type>),
  Function(Function),
  Union(Vec<Type>),
  Existential(Existential),
}
impl Type {
  pub const NULL: Self = Self::Literal(Literal::Null);

  pub fn string_or_number() -> Self {
    Self::Union(vec![
      Self::Literal(Literal::String),
      Self::Literal(Literal::Number),
    ])
  }

  pub fn boolean() -> Self {
    Self::Union(vec![
      Self::Literal(Literal::True),
      Self::Literal(Literal::False),
    ])
  }

  pub fn truthiness(&self) -> Truthiness {
    if self.is_truthy() {
      Truthiness::True
    } else if self.is_falsy() {
      Truthiness::False
    } else {
      Truthiness::Unknown
    }
  }

  pub fn is_truthy(&self) -> bool {
    match self {
      Self::Literal(Literal::True) | Self::Function(_) => true,
      Self::Union(a) => a.iter().all(Self::is_truthy),
      _ => false,
    }
  }

  pub fn is_falsy(&self) -> bool {
    match self {
      Self::Literal(Literal::False | Literal::Null) => true,
      Self::Union(a) => a.iter().all(Self::is_falsy),
      _ => false,
    }
  }

  pub fn is_subtype_of(&self, b: &Self) -> bool {
    match (self, b) {
      (_, Self::Any) => true,
      (_, Self::Never) => false,
      (a, b) if a == b => true,

      (Self::List(a), Self::List(b)) | (Self::Set(a), Self::Set(b)) => a.is_subtype_of(b),
      (Self::Dict(a, b), Self::Dict(c, d)) => a.is_subtype_of(c) && b.is_subtype_of(d),

      (Self::Union(a), b) => a.iter().all(|a| a.is_subtype_of(b)),
      (a, Self::Union(b)) => b.iter().any(|b| a.is_subtype_of(b)),

      (Self::Function(a), Self::Function(b)) => {
        a.parameters.len() == b.parameters.len()
          && a
            .parameters
            .iter()
            .zip(b.parameters.iter())
            .all(|(a, b)| b.is_subtype_of(a))
          && a.return_type.is_subtype_of(&b.return_type)
      }

      (_, _) => false,
    }
  }

  pub fn union(self, b: Self) -> Self {
    match (self, b) {
      (a, Self::Never) | (Self::Never, a) => a,
      (_, Self::Any) | (Self::Any, _) => Self::Any,

      (a, b) if a.is_subtype_of(&b) => b,
      (a, b) if b.is_subtype_of(&a) => a,

      (Self::Union(mut a), Self::Union(b)) => {
        for b in b {
          if a.iter().any(|a| b.is_subtype_of(a)) {
            a.push(b);
          }
        }
        Self::Union(a)
      }
      (a, Self::Union(mut b)) | (Self::Union(mut b), a) => {
        if !b.iter().any(|b| a.is_subtype_of(b)) {
          b.push(a);
        }
        Self::Union(b)
      }

      (a, b) => Self::Union(vec![a, b]),
    }
  }

  pub fn narrow(self, b: &Self) -> Self {
    match (self, b) {
      (a, b) if a == *b => Self::Never,
      (Self::Union(a), Self::Union(b)) => {
        let mut remaining: Vec<_> = a
          .into_iter()
          .filter(|a| a != &Self::Never && b.iter().all(|b| b != a))
          .collect();

        match remaining.len() {
          0 => Self::Never,
          1 => std::mem::take(&mut remaining[0]),
          _ => Self::Union(remaining),
        }
      }
      (Self::Union(a), b) => {
        let mut remaining: Vec<_> = a
          .into_iter()
          .filter(|a| a != b && a != &Self::Never)
          .collect();

        match remaining.len() {
          0 => Self::Never,
          1 => std::mem::take(&mut remaining[0]),
          _ => Self::Union(remaining),
        }
      }
      (x, _) => x,
    }
  }

  pub fn uplevel_boolean(self) -> Self {
    if let Self::Literal(Literal::True | Literal::False) = self {
      Self::boolean()
    } else {
      self
    }
  }

  pub fn apply_context(self, context: &Context) -> Self {
    match self {
      Self::Existential(a) => context.lookup(a).unwrap_or(Self::Existential(a)),

      Self::List(a) => Self::List(a.apply_context(context).into()),
      Self::Set(a) => Self::Set(a.apply_context(context).into()),

      Self::Function(a) => Self::Function(Function {
        catch_all: a.catch_all,
        parameters: a
          .parameters
          .into_iter()
          .map(|ty| ty.apply_context(context))
          .collect(),
        return_type: a.return_type.apply_context(context).into(),
      }),

      Self::Union(a) => Self::Union(a.into_iter().map(|ty| ty.apply_context(context)).collect()),

      _ => self,
    }
  }
}

impl Default for Type {
  fn default() -> Self {
    Self::NULL
  }
}

impl Display for Type {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Any => write!(f, "any"),
      Self::Never => write!(f, "never"),
      Self::Literal(literal) => write!(f, "{literal}"),
      Self::List(ty) => write!(f, "{ty}[]"),
      Self::Set(ty) => write!(f, "set({ty})"),
      Self::Dict(key, values) => write!(f, "set({key}, {values})"),
      Self::Function(func) => write!(
        f,
        "({}) -> {}",
        func
          .parameters
          .iter()
          .map(ToString::to_string)
          .collect::<Vec<_>>()
          .join(", "),
        func.return_type
      ),
      Self::Union(types) => write!(
        f,
        "{}",
        types
          .iter()
          .map(ToString::to_string)
          .collect::<Vec<_>>()
          .join(" | ")
      ),
      Self::Existential(x) => write!(f, "<{x}>"),
    }
  }
}
