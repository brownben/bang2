mod functions;
mod indexing;

pub use functions::{Arity, Function, NativeFunction};
pub use indexing::{calculate_index, Index};
use std::{
  cell::RefCell,
  fmt::{self, Display},
  rc::Rc,
  str,
};

#[derive(Clone)]
pub enum Value {
  Null,
  Boolean(bool),
  Number(f64),
  String(Rc<str>),
  Function(Rc<Function>),
  NativeFunction(Rc<NativeFunction>),
  List(Rc<RefCell<Vec<Value>>>),
}

impl Value {
  pub fn is_falsy(&self) -> bool {
    match self {
      Self::Boolean(value) => !value,
      Self::Null => true,
      Self::Number(value) => (value - 0.0).abs() < f64::EPSILON,
      Self::String(value) => value.is_empty(),
      Self::Function(_) | Self::NativeFunction(_) => false,
      Self::List(value) => value.borrow().is_empty(),
    }
  }

  pub fn get_type(&self) -> &'static str {
    match self {
      Self::Null => "null",
      Self::String(_) => "string",
      Self::Number(_) => "number",
      Self::Boolean(_) => "boolean",
      Self::Function(_) | Self::NativeFunction(_) => "function",
      Self::List(_) => "list",
    }
  }
}

impl PartialEq for Value {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::Boolean(value), Self::Boolean(other)) => value == other,
      (Self::Null, Self::Null) => true,
      (Self::Number(value), Self::Number(other)) => {
        value == other || (value - other).abs() < f64::EPSILON
      }
      (Self::String(value), Self::String(other)) => value.eq(other),
      (Self::Function(value), Self::Function(other)) => Rc::ptr_eq(value, other),
      (Self::NativeFunction(value), Self::NativeFunction(other)) => Rc::ptr_eq(value, other),
      (Self::List(value), Self::List(other)) => {
        let a = value.borrow();
        let b = other.borrow();
        a.len() == b.len() && a.iter().zip(b.iter()).all(|(a, b)| a == b)
      }
      _ => false,
    }
  }
}

impl Display for Value {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Self::Null => write!(f, "null"),
      Self::Boolean(value) => write!(f, "{}", value),
      Self::Number(value) => write!(f, "{}", value),
      Self::String(value) => write!(f, "'{}'", value),
      Self::Function(value) => write!(f, "<function {}>", value.name),
      Self::NativeFunction(value) => write!(f, "<function {}>", value.name),
      Self::List(value) => write!(
        f,
        "[{}]",
        value
          .borrow()
          .iter()
          .map(ToString::to_string)
          .collect::<Vec<String>>()
          .join(", ")
      ),
    }
  }
}

impl From<bool> for Value {
  fn from(value: bool) -> Self {
    Self::Boolean(value)
  }
}
impl From<i32> for Value {
  fn from(value: i32) -> Self {
    Self::Number(f64::from(value))
  }
}
impl From<f64> for Value {
  fn from(value: f64) -> Self {
    Self::Number(value)
  }
}
impl From<usize> for Value {
  fn from(value: usize) -> Self {
    #[allow(clippy::cast_precision_loss)]
    // used by builtins for lengths, if larger allow rounding as that is to be expected with number type
    Self::Number(value as f64)
  }
}

impl From<String> for Value {
  fn from(value: String) -> Self {
    Self::String(Rc::from(value))
  }
}
impl From<&str> for Value {
  fn from(value: &str) -> Self {
    Self::String(Rc::from(value))
  }
}
impl From<char> for Value {
  fn from(value: char) -> Self {
    Self::from(value.to_string())
  }
}
impl From<Vec<u8>> for Value {
  fn from(value: Vec<u8>) -> Self {
    str::from_utf8(&value).into()
  }
}

impl From<Function> for Value {
  fn from(value: Function) -> Self {
    Self::Function(Rc::from(value))
  }
}
impl From<NativeFunction> for Value {
  fn from(value: NativeFunction) -> Self {
    Self::NativeFunction(Rc::from(value))
  }
}

impl From<Vec<Self>> for Value {
  fn from(value: Vec<Self>) -> Self {
    Self::List(Rc::from(RefCell::new(value)))
  }
}

impl From<()> for Value {
  fn from(_: ()) -> Self {
    Self::Null
  }
}
impl<T: Into<Self>> From<Option<T>> for Value {
  fn from(value: Option<T>) -> Self {
    match value {
      Some(value) => value.into(),
      None => Self::Null,
    }
  }
}
impl<T: Into<Self>, E> From<Result<T, E>> for Value {
  fn from(value: Result<T, E>) -> Self {
    match value {
      Ok(value) => value.into(),
      Err(_) => Self::Null,
    }
  }
}

#[cfg(test)]
mod test {
  use super::{Function, NativeFunction, Value};

  #[test]
  fn displays_correctly() {
    assert_eq!(Value::from("hello").to_string(), "'hello'");
    assert_eq!(Value::from(Vec::new() as Vec<u8>).to_string(), "''");
    assert_eq!(Value::from(true).to_string(), "true");
    assert_eq!(Value::from(false).to_string(), "false");
    assert_eq!(Value::from(()).to_string(), "null");
    assert_eq!(Value::from(Vec::new() as Vec<Value>).to_string(), "[]");
    assert_eq!(
      Value::from(vec![Value::from("hello")]).to_string(),
      "['hello']"
    );
    assert_eq!(
      Value::from(Function {
        name: "hello".into(),
        arity: 0.into(),
        start: 0
      })
      .to_string(),
      "<function hello>"
    );

    assert_eq!(
      Value::from(NativeFunction {
        name: "native",
        arity: 0.into(),
        func: |_| Value::Null
      })
      .to_string(),
      "<function native>"
    );
  }
}
