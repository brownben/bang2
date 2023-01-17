use super::Span;

#[derive(Clone, Debug)]
pub struct TypeExpression<'s> {
  pub type_: Type<'s>,
  pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Type<'s> {
  Named(&'s str),
  Parameter(&'s str, Vec<TypeExpression<'s>>),
  Union(Box<TypeExpression<'s>>, Box<TypeExpression<'s>>),
  Function(Box<TypeExpression<'s>>, Vec<TypeExpression<'s>>),
  Optional(Box<TypeExpression<'s>>),
  Group(Box<TypeExpression<'s>>),
  List(Box<TypeExpression<'s>>),
  WithGeneric(Vec<&'s str>, Box<TypeExpression<'s>>),
}

macro_rules! types {
    ($type:ident $struct:tt, ($start:expr, $end:expr)) => {{
      let start = $start;
      let end = $end;

      TypeExpression {
        type_: Type::$type $struct,
        span: Span { start: start.start, end: end.end }
      }
    }};

    ($type:ident $struct:tt, $range:expr) => {
      types!($type $struct, ($range, $range))
    };
  }
pub(crate) use types;
