use crate::chunk::Chunk;

use std::rc::Rc;

#[derive(Debug)]
pub struct Function {
  pub arity: u8,
  pub chunk: Chunk,
  pub name: Rc<str>,
}

impl Function {
  pub fn script(chunk: Chunk) -> Rc<Self> {
    Rc::new(Self {
      arity: 0,
      chunk,
      name: Rc::from(String::from("Main")),
    })
  }
}

#[derive(Debug, Clone)]
pub enum Value {
  Null,
  Boolean(bool),
  Number(f64),
  String(Rc<str>),
  Function(Rc<Function>),
}

impl Value {
  pub fn is_number(&self) -> bool {
    matches!(self, Value::Number(_))
  }

  pub fn is_null(&self) -> bool {
    matches!(self, Value::Null)
  }

  pub fn is_string(&self) -> bool {
    matches!(self, Value::String(_))
  }

  pub fn is_function(&self) -> bool {
    matches!(self, Value::Function(_))
  }

  pub fn get_number_value(&self) -> f64 {
    match self {
      Value::Number(number) => *number,
      _ => 0.0,
    }
  }

  pub fn get_string_value(&self) -> Rc<str> {
    match self {
      Value::String(string) => string.clone(),
      _ => Rc::from(""),
    }
  }

  pub fn get_function_value(&self) -> Option<&Rc<Function>> {
    match self {
      Value::Function(func) => Some(func),
      _ => None,
    }
  }

  pub fn is_falsy(&self) -> bool {
    match self {
      Value::Boolean(value) => !value,
      Value::Null => true,
      Value::Number(value) => (*value - 0.0).abs() < f64::EPSILON,
      Value::String(value) => value.is_empty(),
      Value::Function(_) => false,
    }
  }

  pub fn equals(&self, other: &Self) -> bool {
    match (self, other) {
      (Value::Boolean(value), Value::Boolean(other)) => *value == *other,
      (Value::Null, Value::Null) => true,
      (Value::Number(value), Value::Number(other)) => (*value - *other).abs() < f64::EPSILON,
      (Value::String(value), Value::String(other)) => value.eq(other),
      (Value::Function(value), Value::Function(other)) => Rc::ptr_eq(value, other),
      _ => false,
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
    if value.is_nan() {
      Self::Null
    } else {
      Self::Number(value)
    }
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

impl From<Rc<str>> for Value {
  fn from(value: Rc<str>) -> Self {
    Self::String(value)
  }
}

impl From<Function> for Value {
  fn from(value: Function) -> Self {
    Self::Function(Rc::from(value))
  }
}

impl std::fmt::Display for Value {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      Value::Boolean(value) => write!(f, "{}", value),
      Value::Null => write!(f, "null"),
      Value::Number(value) => write!(f, "{}", value),
      Value::String(value) => write!(f, "{}", value),
      Value::Function(value) => write!(f, "<function {}>", value.name),
    }
  }
}
