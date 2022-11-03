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

  pub fn is_possibly_cyclic(&self) -> bool {
    match self {
      Self::String(_) | Self::Function(_) | Self::NativeFunction(_) | Self::Closure(_) => false,
      Self::List(_) | Self::Set(_) | Self::Dict(_) => true,
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

  pub fn equals(a: &Value, b: &Value, seen: &mut HashSet<u64>) -> bool {
    if !a.is_object() || !b.is_object() {
      return false;
    }

    match (&*a.as_object(), &*b.as_object()) {
      (Self::String(value), Self::String(other)) => value == other,
      (Self::Function(value), Self::Function(other)) => value == other,
      // Native Function and Closure are compared by pointer in Value::eq
      (Self::Set(value), Self::Set(other)) => *value.borrow() == *other.borrow(),
      (Self::List(value), Self::List(other)) => {
        value.as_ptr() == other.as_ptr() || {
          let value = value.borrow();
          let other = other.borrow();

          if value.len() != other.len() {
            return false;
          }

          value
            .iter()
            .zip(other.iter())
            .all(|(a, b)| Value::equals(a, b, &mut seen.clone()))
        }
      }
      (Self::Dict(value), Self::Dict(other)) => {
        let value = value.borrow();
        let other = other.borrow();

        if value.len() != other.len() {
          return false;
        }

        value.iter().all(|(key, value)| {
          other
            .get(key)
            .map_or(false, |v| Value::equals(value, v, &mut seen.clone()))
        })
      }
      _ => false,
    }
  }

  pub fn format(
    f: &mut fmt::Formatter<'_>,
    value: &Self,
    seen: &mut HashSet<u64>,
    debug: bool,
  ) -> fmt::Result {
    match value {
      Self::String(value) if debug => write!(f, "'{value}'"),
      Self::String(value) => write!(f, "{value}"),
      Self::Function(value) => write!(f, "<function {}>", value.name),
      Self::NativeFunction(value) => write!(f, "<function {}>", value.name),
      Self::Closure(value) => write!(f, "<function {}>", value.func.name),
      Self::List(value) => {
        write!(f, "[")?;
        value
          .borrow()
          .iter()
          .enumerate()
          .try_for_each(|(index, item)| {
            if seen.contains(&item.as_bytes()) {
              write!(f, "...")?;
            } else {
              Value::format(f, item, &mut seen.clone())?;
            }

            if index != value.borrow().len() - 1 {
              write!(f, ", ")?;
            }

            Ok(())
          })?;
        write!(f, "]")
      }
      Self::Set(value) => {
        write!(f, "set(")?;
        value
          .borrow()
          .iter()
          .enumerate()
          .try_for_each(|(index, item)| {
            if seen.contains(&item.as_bytes()) {
              write!(f, "...")?;
            } else {
              Value::format(f, item, &mut seen.clone())?;
            }

            if index != value.borrow().len() - 1 {
              write!(f, ", ")?;
            }

            Ok(())
          })?;
        write!(f, ")")
      }
      Self::Dict(value) => {
        write!(f, "{{ ")?;
        value
          .borrow()
          .iter()
          .enumerate()
          .try_for_each(|(index, (k, v))| {
            if seen.contains(&k.as_bytes()) {
              write!(f, "...")?;
            } else {
              Value::format(f, k, seen)?;
            }

            write!(f, ": ")?;

            if seen.contains(&v.as_bytes()) {
              write!(f, "...")?;
            } else {
              Value::format(f, v, &mut seen.clone())?;
            }

            if index != value.borrow().len() - 1 {
              write!(f, ", ")?;
            }

            Ok(())
          })?;
        write!(f, " }}")
      }
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
