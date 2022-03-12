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
      Self::String(string) => string.clone(),
      _ => Rc::from(""),
    }
  }

  pub fn is_falsy(&self) -> bool {
    match self {
      Self::Boolean(value) => !value,
      Self::Null => true,
      Self::Number(value) => (value - 0.0).abs() < f64::EPSILON,
      Self::String(value) => value.is_empty(),
      Self::Function(_) | Self::NativeFunction(_) => false,
    }
  }

  pub fn get_type(&self) -> &'static str {
    match self {
      Self::Null => "null",
      Self::String(_) => "string",
      Self::Number(_) => "number",
      Self::Boolean(_) => "boolean",
      Self::Function(_) | Self::NativeFunction(_) => "function",
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
      (Self::Boolean(value), Self::Boolean(other)) => value == other,
      (Self::Null, Self::Null) => true,
      (Self::Number(value), Self::Number(other)) => (value - other).abs() < f64::EPSILON,
      (Self::String(value), Self::String(other)) => value.eq(other),
      (Self::Function(value), Self::Function(other)) => Rc::ptr_eq(value, other),
      (Self::NativeFunction(value), Self::NativeFunction(other)) => Rc::ptr_eq(value, other),
      _ => false,
    }
  }
}

impl Display for Value {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      Self::Null => write!(f, "null"),
      Self::Boolean(value) => write!(f, "{}", value),
      Self::Number(value) => write!(f, "{}", value),
      Self::String(value) => write!(f, "'{}'", value),
      Self::Function(value) => write!(f, "<function {}>", value.name),
      Self::NativeFunction(value) => write!(f, "<function {}>", value.name),
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
