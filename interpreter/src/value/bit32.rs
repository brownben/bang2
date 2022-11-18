pub use super::Object;
use std::{cell::RefCell, mem, ptr, rc::Rc};

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(C)]
pub struct Inner {
  tag: usize,
  pointer: *const Object,
}

pub struct Value(Inner);

impl Value {
  pub const NULL: Self = Self(NULL);
  pub const TRUE: Self = Self(TRUE);
  pub const FALSE: Self = Self(FALSE);

  pub fn is_object(&self) -> bool {
    self.0.tag == IS_PTR
  }
  pub fn as_object(&self) -> &Object {
    unsafe { &*self.0.pointer }
  }

  pub fn is_number(&self) -> bool {
    (self.0.tag & IS_NUMBER) != IS_NUMBER
  }
  pub fn as_number(&self) -> f64 {
    unsafe { mem::transmute(self.0) }
  }

  pub fn as_bytes(&self) -> u64 {
    unsafe { mem::transmute(self.0) }
  }

  #[must_use]
  pub fn allocate(self) -> Self {
    let memory = Rc::new(RefCell::new(self));
    let pointer = Rc::into_raw(memory);

    Self(Inner {
      tag: IS_ALLOCATED,
      pointer: pointer.cast::<Object>(),
    })
  }
  pub fn is_allocated(&self) -> bool {
    self.0.tag == IS_ALLOCATED
  }
  pub fn as_allocated(&self) -> Rc<RefCell<Self>> {
    let pointer = self.0.pointer.cast::<RefCell<Self>>();

    unsafe { Rc::increment_strong_count(pointer) };
    unsafe { Rc::from_raw(pointer) }
  }
}

impl Clone for Value {
  fn clone(&self) -> Self {
    if self.is_object() {
      unsafe { Rc::increment_strong_count(self.0.pointer) };
    } else if self.is_allocated() {
      let pointer = self.0.pointer.cast::<RefCell<Self>>();
      unsafe { Rc::increment_strong_count(pointer) };
    }

    Self(self.0)
  }
}

impl Drop for Value {
  fn drop(&mut self) {
    if self.is_object() {
      let pointer = self.0.pointer;
      unsafe { Rc::decrement_strong_count(pointer) };
    } else if self.is_allocated() {
      let pointer = self.0.pointer.cast::<RefCell<Self>>();
      unsafe { Rc::decrement_strong_count(pointer) };
    }
  }
}

impl From<f64> for Value {
  fn from(value: f64) -> Self {
    let [byte1, byte2]: [usize; 2] = unsafe { mem::transmute(value) };
    Self(Inner {
      tag: byte1,
      pointer: ptr::invalid(byte2),
    })
  }
}

impl From<Rc<Object>> for Value {
  fn from(value: Rc<Object>) -> Self {
    let pointer = Rc::into_raw(value);
    Self(Inner {
      tag: IS_PTR,
      pointer,
    })
  }
}

const IS_PTR: usize = 0b1111_1111_1111_1111_1111_1111_1111_1110;
const IS_ALLOCATED: usize = 0b1111_1111_1111_1111_1111_1111_1111_1100;
const IS_NUMBER: usize = 0b0111_1111_1111_1000_0000_0000_0000_0000;

pub const TRUE: Inner = Inner {
  tag: 0b1111_1111_1111_1100_0000_0000_0000_0000,
  pointer: ptr::invalid(0),
};
pub const FALSE: Inner = Inner {
  tag: 0b1111_1111_1111_1110_0000_0000_0000_0000,
  pointer: ptr::invalid(0),
};
pub const NULL: Inner = Inner {
  tag: 0b1111_1111_1111_1111_0000_0000_0000_0000,
  pointer: ptr::invalid(0),
};
