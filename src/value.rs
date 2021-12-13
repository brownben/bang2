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

impl std::fmt::Debug for NativeFunction {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("NativeFunction")
      .field("name", &self.name)
      .field("arity", &self.arity)
      .finish()
  }
}

#[derive(Debug, Clone)]
pub enum Value {
  Null,
  Boolean(bool),
  Number(f64),
  String(Rc<str>),
  Function(Rc<Function>),
  NativeFunction(Rc<NativeFunction>),
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

  pub fn is_callable(&self) -> bool {
    matches!(self, Value::Function(_) | Value::NativeFunction(_))
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

  pub fn is_falsy(&self) -> bool {
    match self {
      Value::Boolean(value) => !value,
      Value::Null => true,
      Value::Number(value) => (*value - 0.0).abs() < f64::EPSILON,
      Value::String(value) => value.is_empty(),
      Value::Function(_) | Value::NativeFunction(_) => false,
    }
  }

  pub fn equals(&self, other: &Self) -> bool {
    match (self, other) {
      (Value::Boolean(value), Value::Boolean(other)) => *value == *other,
      (Value::Null, Value::Null) => true,
      (Value::Number(value), Value::Number(other)) => (*value - *other).abs() < f64::EPSILON,
      (Value::String(value), Value::String(other)) => value.eq(other),
      (Value::Function(value), Value::Function(other)) => Rc::ptr_eq(value, other),
      (Value::NativeFunction(value), Value::NativeFunction(other)) => Rc::ptr_eq(value, other),
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

impl From<NativeFunction> for Value {
  fn from(value: NativeFunction) -> Self {
    Self::NativeFunction(Rc::from(value))
  }
}

impl std::fmt::Display for Value {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      Value::Boolean(value) => write!(f, "{}", value),
      Value::Null => write!(f, "null"),
      Value::Number(value) => write!(f, "{}", value),
      Value::String(value) => write!(f, "'{}'", value),
      Value::Function(value) => write!(f, "<function {}>", value.name),
      Value::NativeFunction(value) => write!(f, "<function {}>", value.name),
    }
  }
}
