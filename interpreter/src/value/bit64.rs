pub use super::Object;
use std::{mem, ptr, rc::Rc};

pub struct Value(*const Object);

impl Value {
  pub const NULL: Self = Self(NULL);
  pub const TRUE: Self = Self(TRUE);
  pub const FALSE: Self = Self(FALSE);

  pub fn is_object(&self) -> bool {
    (self.0.addr() & TO_STORED) == TO_STORED && self.0 != NULL
  }
  pub fn as_object(&self) -> Rc<Object> {
    let pointer = self.0.map_addr(|ptr| ptr & FROM_STORED);
    unsafe { Rc::increment_strong_count(pointer) };
    unsafe { Rc::from_raw(pointer) }
  }

  pub fn is_number(&self) -> bool {
    (self.0.addr() & IS_NUMBER) != IS_NUMBER
  }
  pub fn as_number(&self) -> f64 {
    unsafe { mem::transmute(self.0) }
  }
}

impl Clone for Value {
  fn clone(&self) -> Self {
    if self.is_object() {
      Self::from(self.as_object())
    } else {
      Self(self.0)
    }
  }
}

impl Drop for Value {
  fn drop(&mut self) {
    if self.is_object() {
      let pointer = self.0.map_addr(|ptr| ptr & FROM_STORED);
      unsafe { Rc::from_raw(pointer) };
    }
  }
}

impl From<f64> for Value {
  fn from(value: f64) -> Self {
    #[allow(clippy::cast_possible_truncation)] // as 64 bit code, usize == u64
    Self(ptr::invalid(value.to_bits() as usize))
  }
}

impl From<Rc<Object>> for Value {
  fn from(value: Rc<Object>) -> Self {
    let pointer = Rc::into_raw(value);
    Self(pointer.map_addr(|ptr| ptr | TO_STORED))
  }
}

const TO_STORED: usize =
  0b1111_1111_1111_1111_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000;
const FROM_STORED: usize =
  0b0000_0000_0000_0000_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111;
const IS_NUMBER: usize =
  0b0111_1111_1111_1100_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000;

pub const TRUE: *const Object =
  ptr::invalid(0b1111_1111_1111_1101_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000);
pub const FALSE: *const Object =
  ptr::invalid(0b1111_1111_1111_1110_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000);
pub const NULL: *const Object =
  ptr::invalid(0b1111_1111_1111_1111_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000);
