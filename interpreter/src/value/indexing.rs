use super::{Object, Value};

pub enum GetResult {
  Found(Value),
  NotFound,
  NotSupported,
}
impl From<Option<Value>> for GetResult {
  fn from(value: Option<Value>) -> Self {
    if let Some(value) = value {
      Self::Found(value)
    } else {
      Self::NotFound
    }
  }
}

pub enum SetResult {
  Set,
  NotFound,
  NotSupported,
}

pub trait Index {
  fn get_property(&self, _index: Value) -> GetResult {
    GetResult::NotSupported
  }
  fn set_property(&mut self, _index: Value, _value: Value) -> SetResult {
    SetResult::NotSupported
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
  fn get_property(&self, index: Value) -> GetResult {
    if self.is_object() {
      match &*self.as_object() {
        Object::List(list) => list.borrow().get_property(index),
        Object::String(string) => string.get_property(index),
        _ => GetResult::NotSupported,
      }
    } else {
      GetResult::NotSupported
    }
  }

  fn set_property(&mut self, index: Value, value: Value) -> SetResult {
    if self.is_object() {
      match &*self.as_object() {
        Object::List(list) => list.borrow_mut().set_property(index, value),
        _ => SetResult::NotSupported,
      }
    } else {
      SetResult::NotSupported
    }
  }
}
impl Index for String {
  fn get_property(&self, index: Value) -> GetResult {
    if index.is_number() {
      self
        .chars()
        .nth(calculate_index(index.as_number(), self.len()))
        .map(Value::from)
        .into()
    } else {
      GetResult::NotFound
    }
  }
}
impl Index for Vec<Value> {
  fn get_property(&self, index: Value) -> GetResult {
    if index.is_number() {
      let index = calculate_index(index.as_number(), self.len());
      self.get(index).cloned().into()
    } else {
      GetResult::NotFound
    }
  }

  fn set_property(&mut self, index: Value, value: Value) -> SetResult {
    if index.is_number() {
      let index = calculate_index(index.as_number(), self.len());
      if index < self.len() {
        self[index] = value;
        return SetResult::Set;
      }
      return SetResult::NotFound;
    }

    SetResult::NotSupported
  }
}
