use std::{fmt::Display, rc::Rc};

use crate::chunk::Chunk;

pub struct Function {
  pub name: String,
  pub arity: u8,
  pub chunk: Chunk,
}
impl Function {
  pub fn script(chunk: Chunk) -> Rc<Self> {
    Rc::new(Self {
      arity: 0,
      chunk,
      name: String::new(),
    })
  }
}

pub struct NativeFunction {
  pub name: String,
  pub arity: u8,
  pub func: fn(args: &[Value]) -> Value,
}
impl NativeFunction {
  pub fn create(name: &str, arity: u8, func: fn(args: &[Value]) -> Value) -> Value {
    Value::from(Self {
      name: name.to_string(),
      arity,
      func,
    })
  }
}

#[derive(Clone)]
pub enum Value {
  Null,
  Boolean(bool),
  Number(f64),
  String(Rc<str>),
  Function(Rc<Function>),
  NativeFunction(Rc<NativeFunction>),
}

impl Value {
  pub fn as_str(&self) -> Rc<str> {
    match self {
      Value::String(string) => string.clone(),
      _ => Rc::from(""),
    }
  }

  pub fn is_falsy(&self) -> bool {
    match self {
      Value::Boolean(value) => !value,
      Value::Null => true,
      Value::Number(value) => (value - 0.0).abs() < f64::EPSILON,
      Value::String(value) => value.is_empty(),
      Value::Function(_) | Value::NativeFunction(_) => false,
    }
  }

  pub fn parse_number(string: &str) -> Value {
    let value: f64 = string.replace('_', "").parse().unwrap();

    Value::from(value)
  }
}

impl PartialEq for Value {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Value::Boolean(value), Value::Boolean(other)) => value == other,
      (Value::Null, Value::Null) => true,
      (Value::Number(value), Value::Number(other)) => (value - other).abs() < f64::EPSILON,
      (Value::String(value), Value::String(other)) => value.eq(other),
      (Value::Function(value), Value::Function(other)) => Rc::ptr_eq(value, other),
      (Value::NativeFunction(value), Value::NativeFunction(other)) => Rc::ptr_eq(value, other),
      _ => false,
    }
  }
}

impl Display for Value {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      Value::Null => write!(f, "null"),
      Value::Boolean(value) => write!(f, "{}", value),
      Value::Number(value) => write!(f, "{}", value),
      Value::String(value) => write!(f, "'{}'", value),
      Value::Function(value) => write!(f, "<function {}>", value.name),
      Value::NativeFunction(value) => write!(f, "<function {}>", value.name),
    }
  }
}

impl From<bool> for Value {
  fn from(value: bool) -> Self {
    Self::Boolean(value)
  }
}
impl From<f64> for Value {
  fn from(value: f64) -> Self {
    Self::Number(value)
  }
}
impl From<String> for Value {
  fn from(value: String) -> Self {
    Self::String(Rc::from(value))
  }
}
impl From<&str> for Value {
  fn from(value: &str) -> Self {
    Self::String(Rc::from(String::from(value)))
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
