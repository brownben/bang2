mod functions;
pub mod indexing;
mod objects;

#[cfg(target_pointer_width = "32")]
mod bit32;
#[cfg(target_pointer_width = "32")]
pub use bit32::Value;
#[cfg(target_pointer_width = "32")]
use bit32::{FALSE, NULL, TRUE};

#[cfg(target_pointer_width = "64")]
mod bit64;
#[cfg(target_pointer_width = "64")]
pub use bit64::Value;
#[cfg(target_pointer_width = "64")]
use bit64::{FALSE, NULL, TRUE};

#[cfg(test)]
mod test;

use crate::collections::HashSet;
pub use functions::{Arity, Closure, ClosureKind, Function, NativeFunction};
pub use indexing::calculate_index;
pub use objects::Object;
use smartstring::alias::String;
use std::{
  collections::BTreeSet,
  fmt::{self, Write},
  hash,
  rc::Rc,
};

impl Value {
  pub fn is_falsy(&self) -> bool {
    match self {
      Self(TRUE) => false,
      Self(FALSE | NULL) => true,
      a if a.is_number() => (a.as_number() - 0.0).abs() < f64::EPSILON,
      b => b.as_object().is_falsy(),
    }
  }
  pub fn get_type(&self) -> &'static str {
    match self {
      Self(TRUE | FALSE) => "boolean",
      Self(NULL) => "null",
      a if a.is_number() => "number",
      b => b.as_object().get_type(),
    }
  }

  pub fn to_string(&self) -> String {
    let mut string = String::new();
    write!(string, "{self}").expect("No errors in Display trait");
    string
  }

  pub fn equals(a: &Self, b: &Self, seen: &mut BTreeSet<u64>) -> bool {
    if a.as_bytes() == b.as_bytes() {
      return true;
    }

    if a.is_number() && b.is_number() {
      let a = a.as_number();
      let b = b.as_number();
      return (a - b).abs() < f64::EPSILON;
    }

    if seen.contains(&a.as_bytes())
      || seen.contains(&b.as_bytes())
      || !a.is_object()
      || !b.is_object()
    {
      return false;
    }

    if a.as_object().is_possibly_cyclic() {
      seen.insert(a.as_bytes());
    }
    if b.as_object().is_possibly_cyclic() {
      seen.insert(b.as_bytes());
    }

    Object::equals(a.as_object(), b.as_object(), seen)
  }

  fn format(f: &mut fmt::Formatter, value: &Self, seen: &mut HashSet<u64>) -> fmt::Result {
    match value {
      a if a.is_object() => {
        let obj = &a.as_object();
        if obj.is_possibly_cyclic() {
          seen.insert(a.as_bytes());
        }
        Object::format(f, obj, seen, true)
      }
      b if b.is_allocated() => write!(f, "pointer"),
      c => write!(f, "{c}"),
    }
  }
}

impl PartialEq for Value {
  fn eq(&self, other: &Self) -> bool {
    if self.as_bytes() == other.as_bytes() {
      return true;
    }

    if self.is_number() && other.is_number() {
      let a = self.as_number();
      let b = other.as_number();
      return (a - b).abs() < f64::EPSILON;
    }

    if !self.is_object() || !other.is_object() {
      return false;
    }

    match (self.as_object(), other.as_object()) {
      (Object::String(value), Object::String(other)) => value == other,
      _ => false,
    }
  }
}
impl Eq for Value {}

impl hash::Hash for Value {
  fn hash<H: hash::Hasher>(&self, state: &mut H) {
    match self {
      Self(NULL) => 0,
      Self(TRUE) => 1,
      Self(FALSE) => 2,
      value if value.is_number() => 3,
      _ => 4,
    }
    .hash(state);

    match self {
      a if a.is_number() => a.as_number().to_le_bytes().hash(state),
      b if b.is_object() => b.as_object().hash(state),
      _ => {}
    }
  }
}

impl fmt::Display for Value {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Self(NULL) => write!(f, "null"),
      Self(TRUE) => write!(f, "true"),
      Self(FALSE) => write!(f, "false"),
      a if a.is_number() => write!(f, "{}", a.as_number()),
      b => {
        let mut seen = HashSet::default();
        seen.insert(self.as_bytes());

        Object::format(f, b.as_object(), &mut seen, false)
      }
    }
  }
}
impl fmt::Debug for Value {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      a if a.is_object() => {
        let mut seen = HashSet::default();
        seen.insert(self.as_bytes());

        Object::format(f, a.as_object(), &mut seen, true)
      }
      b if b.is_allocated() => write!(f, "pointer"),
      c => write!(f, "{c}"),
    }
  }
}

impl From<()> for Value {
  fn from(_: ()) -> Self {
    Self::NULL
  }
}
impl From<bool> for Value {
  fn from(value: bool) -> Self {
    if value {
      Self::TRUE
    } else {
      Self::FALSE
    }
  }
}
impl From<i32> for Value {
  fn from(value: i32) -> Self {
    Self::from(f64::from(value))
  }
}
impl From<usize> for Value {
  fn from(value: usize) -> Self {
    // used by builtins for lengths
    // if larger allow rounding as that is to be expected with number type
    #[allow(clippy::cast_precision_loss)]
    Self::from(value as f64)
  }
}

impl<T: Into<Object>> From<T> for Value {
  fn from(value: T) -> Self {
    Self::from(Rc::new(value.into()))
  }
}

impl<T: Into<Self>> From<Option<T>> for Value {
  fn from(value: Option<T>) -> Self {
    match value {
      Some(value) => value.into(),
      None => Self::NULL,
    }
  }
}
impl<T: Into<Self>, E> From<Result<T, E>> for Value {
  fn from(value: Result<T, E>) -> Self {
    match value {
      Ok(value) => value.into(),
      Err(_) => Self::NULL,
    }
  }
}
