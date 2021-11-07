#[derive(Debug, Clone)]
pub enum Value {
  Null,
  Boolean(bool),
  Number(f64),
  String(Box<str>),
}

impl Value {
  pub fn is_number(&self) -> bool {
    match self {
      Value::Number(_) => true,
      _ => false,
    }
  }

  pub fn is_null(&self) -> bool {
    match self {
      Value::Null => true,
      _ => false,
    }
  }

  pub fn is_string(&self) -> bool {
    match self {
      Value::String(_) => true,
      _ => false,
    }
  }

  pub fn get_number_value(&self) -> f64 {
    match self {
      Value::Number(number) => *number,
      _ => 0.0,
    }
  }

  pub fn get_string_value(&self) -> String {
    match self {
      Value::String(string) => string.clone().into_string(),
      _ => String::from(""),
    }
  }

  pub fn is_falsy(&self) -> bool {
    match self {
      Value::Boolean(value) => !value,
      Value::Null => true,
      Value::Number(value) => *value == 0 as f64,
      Value::String(value) => value.is_empty(),
    }
  }

  pub fn equals(&self, other: &Value) -> bool {
    match (self, other) {
      (Value::Boolean(value), Value::Boolean(other)) => *value == *other,
      (Value::Null, Value::Null) => true,
      (Value::Number(value), Value::Number(other)) => *value == *other,
      (Value::String(value), Value::String(other)) => *value == *other,
      _ => false,
    }
  }

  pub fn duplicate(&self) -> Self {
    match self {
      Value::Boolean(value) => Value::Boolean(*value),
      Value::Null => Value::Null,
      Value::Number(value) => Value::Number(*value),
      Value::String(value) => Value::String(value.to_string().into_boxed_str()),
    }
  }
}

pub fn print_optional(value: Option<Value>) {
  match value {
    Some(value) => print(value),
    None => print!(""),
  }
}

pub fn print(value: Value) {
  match value {
    Value::Boolean(value) => print!("{}", value),
    Value::Null => print!("null"),
    Value::Number(value) => print!("{}", value),
    Value::String(value) => print!("{}", value),
  }
}

pub fn println(value: Value) {
  match value {
    Value::Boolean(value) => println!("{}", value),
    Value::Null => println!("null"),
    Value::Number(value) => println!("{}", value),
    Value::String(value) => println!("{}", value),
  }
}
