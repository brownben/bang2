use super::Value;
use std::rc::Rc;

pub trait Index {
  fn get_property(&self, _index: Value) -> Option<Value> {
    None
  }
  fn set_property(&mut self, _index: Value, _value: Value) -> bool {
    false
  }
}

#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
pub fn calculate_index(number: f64, length: usize) -> usize {
  let index = number.round().abs() as usize;

  if index > length {
    length
  } else if number < 0.0 {
    length - index
  } else {
    index
  }
}

impl Index for Value {
  fn get_property(&self, index: Value) -> Option<Value> {
    match self {
      Self::List(list) => list.borrow().get_property(index),
      Self::String(string) => string.get_property(index),
      _ => None,
    }
  }

  fn set_property(&mut self, index: Value, value: Value) -> bool {
    match self {
      Self::List(list) => list.borrow_mut().set_property(index, value),
      _ => false,
    }
  }
}
impl Index for Rc<str> {
  fn get_property(&self, index: Value) -> Option<Value> {
    match index {
      Value::Number(n) => self
        .chars()
        .nth(calculate_index(n, self.len()))
        .map(Value::from),
      _ => None,
    }
  }
}
impl Index for Vec<Value> {
  fn get_property(&self, index: Value) -> Option<Value> {
    if let Value::Number(number) = index {
      let index = calculate_index(number, self.len());
      self.get(index).cloned()
    } else {
      None
    }
  }

  fn set_property(&mut self, index: Value, value: Value) -> bool {
    if let Value::Number(number) = index {
      let index = calculate_index(number, self.len());
      if index < self.len() {
        self[index] = value;
        true
      } else {
        false
      }
    } else {
      false
    }
  }
}
