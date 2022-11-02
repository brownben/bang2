use super::{Closure, Function, NativeFunction, Value};
use crate::{HashMap, HashSet};
use smartstring::alias::String;
use std::{cell::RefCell, fmt, hash, mem, ptr, str};

pub enum Object {
  String(String),
  Function(Function),
  NativeFunction(NativeFunction),
  Closure(Closure),
  List(RefCell<Vec<Value>>),
  Set(RefCell<HashSet<Value>>),
  Dict(RefCell<HashMap<Value, Value>>),
}

impl Object {
  pub fn is_falsy(&self) -> bool {
    match self {
      Self::String(value) => value.is_empty(),
      Self::Function(_) | Self::NativeFunction(_) | Self::Closure(_) => false,
      Self::List(value) => value.borrow().is_empty(),
      Self::Set(value) => value.borrow().is_empty(),
      Self::Dict(value) => value.borrow().is_empty(),
    }
  }

  pub fn get_type(&self) -> &'static str {
    match self {
      Self::String(_) => "string",
      Self::Function(_) | Self::NativeFunction(_) | Self::Closure(_) => "function",
      Self::List(_) => "list",
      Self::Set(_) => "set",
      Self::Dict(_) => "dict",
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
      (Self::List(value), Self::List(other)) => *value == *other,
      (Self::Set(value), Self::Set(other)) => *value == *other,
      (Self::Dict(value), Self::Dict(other)) => *value == *other,
      _ => false,
    }
  }
}

impl hash::Hash for Object {
  fn hash<H: hash::Hasher>(&self, state: &mut H) {
    mem::discriminant(self).hash(state);

    match self {
      Self::String(value) => value.hash(state),
      Self::Function(value) => ptr::hash(value, state),
      Self::NativeFunction(value) => ptr::hash(value, state),
      Self::Closure(value) => ptr::hash(value, state),
      Self::List(value) => ptr::hash(value, state),
      Self::Set(value) => ptr::hash(value, state),
      Self::Dict(value) => ptr::hash(value, state),
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
      Self::Set(value) => {
        write!(f, "{{ ")?;
        value
          .borrow()
          .iter()
          .try_for_each(|item| write!(f, "{item:?}, "))?;
        write!(f, "}}")
      }
      Self::Dict(value) => {
        write!(f, "{{ ")?;
        value
          .borrow()
          .iter()
          .try_for_each(|(k, v)| write!(f, "{k:?}: {v:?}, "))?;
        write!(f, "}}")
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
    Self::String(value.into())
  }
}
impl From<char> for Object {
  fn from(value: char) -> Self {
    let mut string = String::new();
    string.extend([value]);
    Self::String(string)
  }
}
impl From<std::string::String> for Object {
  fn from(value: std::string::String) -> Self {
    Self::String(value.into())
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
impl From<HashSet<Value>> for Object {
  fn from(value: HashSet<Value>) -> Self {
    Self::Set(RefCell::new(value))
  }
}
impl From<HashMap<Value, Value>> for Object {
  fn from(value: HashMap<Value, Value>) -> Self {
    Self::Dict(RefCell::new(value))
  }
}
