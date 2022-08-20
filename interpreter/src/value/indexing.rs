use super::{Object, Value};

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
    if self.is_object() {
      match &*self.as_object() {
        Object::List(list) => list.borrow().get_property(index),
        Object::String(string) => string.get_property(index),
        _ => None,
      }
    } else {
      None
    }
  }

  fn set_property(&mut self, index: Value, value: Value) -> bool {
    if self.is_object() {
      match &*self.as_object() {
        Object::List(list) => list.borrow_mut().set_property(index, value),
        _ => false,
      }
    } else {
      false
    }
  }
}
impl Index for String {
  fn get_property(&self, index: Value) -> Option<Value> {
    if index.is_number() {
      self
        .chars()
        .nth(calculate_index(index.as_number(), self.len()))
        .map(Value::from)
    } else {
      None
    }
  }
}
impl Index for Vec<Value> {
  fn get_property(&self, index: Value) -> Option<Value> {
    if index.is_number() {
      let index = calculate_index(index.as_number(), self.len());
      self.get(index).cloned()
    } else {
      None
    }
  }

  fn set_property(&mut self, index: Value, value: Value) -> bool {
    if index.is_number() {
      let index = calculate_index(index.as_number(), self.len());
      if index < self.len() {
        self[index] = value;
        return true;
      }
    }

    false
  }
}
