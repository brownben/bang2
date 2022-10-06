use super::{Closure, Function, NativeFunction, Value};
use std::{cell::RefCell, fmt, ptr, str};

pub enum Object {
  String(String),
  Function(Function),
  NativeFunction(NativeFunction),
  Closure(Closure),
  List(RefCell<Vec<Value>>),
}

impl Object {
  pub fn is_falsy(&self) -> bool {
    match self {
      Self::String(value) => value.is_empty(),
      Self::Function(_) | Self::NativeFunction(_) | Self::Closure(_) => false,
      Self::List(value) => value.borrow().is_empty(),
    }
  }

  pub fn get_type(&self) -> &'static str {
    match self {
      Self::String(_) => "string",
      Self::Function(_) | Self::NativeFunction(_) | Self::Closure(_) => "function",
      Self::List(_) => "list",
    }
  }
}

impl PartialEq for Object {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::String(value), Self::String(other)) => value == other,
      (Self::Function(value), Self::Function(other)) => value == other,
      (Self::NativeFunction(value), Self::NativeFunction(other)) => ptr::eq(value, other),
      (Self::Closure(value), Self::Closure(other)) => ptr::eq(value, other),
      (Self::List(value), Self::List(other)) => {
        let a = value.borrow();
        let b = other.borrow();
        a.len() == b.len() && a.iter().zip(b.iter()).all(|(a, b)| a == b)
      }
      _ => false,
    }
  }
}

impl fmt::Display for Object {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::String(value) => write!(f, "{value}"),
      Self::Function(value) => write!(f, "<function {}>", value.name),
      Self::NativeFunction(value) => write!(f, "<function {}>", value.name),
      Self::Closure(value) => write!(f, "<function {}>", value.func.name),
      Self::List(value) => {
        write!(f, "[")?;
        if let Some((last, elements)) = value.borrow().split_last() {
          elements
            .iter()
            .try_for_each(|item| write!(f, "{item:?}, "))?;
          write!(f, "{last:?}")?;
        }
        write!(f, "]")
      }
    }
  }
}
impl fmt::Debug for Object {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::String(value) => write!(f, "'{value}'"),
      value => write!(f, "{value}"),
    }
  }
}

impl From<String> for Object {
  fn from(value: String) -> Self {
    Self::String(value)
  }
}
impl From<&str> for Object {
  fn from(value: &str) -> Self {
    Self::String(value.to_string())
  }
}
impl From<char> for Object {
  fn from(value: char) -> Self {
    Self::from(value.to_string())
  }
}

impl From<Function> for Object {
  fn from(value: Function) -> Self {
    Self::Function(value)
  }
}
impl From<NativeFunction> for Object {
  fn from(value: NativeFunction) -> Self {
    Self::NativeFunction(value)
  }
}
impl From<Closure> for Object {
  fn from(value: Closure) -> Self {
    Self::Closure(value)
  }
}

impl From<Vec<Value>> for Object {
  fn from(value: Vec<Value>) -> Self {
    Self::List(RefCell::new(value))
  }
}
