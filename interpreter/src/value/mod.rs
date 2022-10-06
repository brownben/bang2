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

pub use functions::{Arity, Closure, Function, NativeFunction};
pub use indexing::calculate_index;
pub use objects::Object;
use std::{fmt, rc::Rc};

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
}

impl PartialEq for Value {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Self(TRUE), Self(TRUE)) | (Self(FALSE), Self(FALSE)) | (Self(NULL), Self(NULL)) => true,
      (a, b) if a.is_number() && b.is_number() => {
        let a = a.as_number();
        let b = b.as_number();
        a == b || (a - b).abs() < f64::EPSILON
      }
      (a, b) if a.is_object() && b.is_object() => a.as_object() == b.as_object(),
      _ => false,
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
      b => write!(f, "{}", b.as_object()),
    }
  }
}
impl fmt::Debug for Value {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      a if a.is_object() => write!(f, "{:?}", a.as_object()),
      b => write!(f, "{b}"),
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
