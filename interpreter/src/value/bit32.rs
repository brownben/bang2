pub use super::Object;
use std::{mem, ptr, rc::Rc};

pub struct Value((usize, *const Object));

impl Value {
  pub const NULL: Self = Self(NULL);
  pub const TRUE: Self = Self(TRUE);
  pub const FALSE: Self = Self(FALSE);

  pub fn is_object(&self) -> bool {
    self.0 .0 == IS_PTR
  }
  pub fn as_object(&self) -> Rc<Object> {
    let pointer = self.0 .1;
    unsafe { Rc::increment_strong_count(pointer) };
    unsafe { Rc::from_raw(pointer) }
  }

  pub fn is_number(&self) -> bool {
    (self.0 .0 & IS_NUMBER) != IS_NUMBER
  }
  pub fn as_number(&self) -> f64 {
    #[allow(clippy::transmute_undefined_repr)] // Assume tuple has no extra padding
    unsafe {
      mem::transmute(self.0)
    }
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
      let pointer = self.0 .1;
      unsafe { Rc::from_raw(pointer) };
    }
  }
}

impl From<f64> for Value {
  fn from(value: f64) -> Self {
    let [byte1, byte2]: [usize; 2] = unsafe { mem::transmute(value) };
    Self((byte1, ptr::invalid(byte2)))
  }
}

impl From<Rc<Object>> for Value {
  fn from(value: Rc<Object>) -> Self {
    let pointer = Rc::into_raw(value);
    Self((IS_PTR, pointer))
  }
}

const IS_PTR: usize = 0b1111_1111_1111_1111_1111_1111_1111_1110;
const IS_NUMBER: usize = 0b0111_1111_1111_1000_0000_0000_0000_0000;

pub const TRUE: (usize, *const Object) =
  (0b1111_1111_1111_1100_0000_0000_0000_0000, ptr::invalid(0));
pub const FALSE: (usize, *const Object) =
  (0b1111_1111_1111_1110_0000_0000_0000_0000, ptr::invalid(0));
pub const NULL: (usize, *const Object) =
  (0b1111_1111_1111_1111_0000_0000_0000_0000, ptr::invalid(0));
