use super::Value;
use crate::chunk::Chunk;
use smallvec::SmallVec;
use smartstring::alias::String;
use std::rc::Rc;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct Arity {
  count: u8,
}
impl Arity {
  pub fn new(count: u8) -> Self {
    Self { count }
  }

  pub fn get_count(self) -> usize {
    usize::from(self.count)
  }

  pub fn check_arg_count(self, provided: u8) -> bool {
    self.count == provided
  }
}
impl From<u8> for Arity {
  fn from(count: u8) -> Self {
    Self { count }
  }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ClosureKind {
  Open,
  Closed,
  Upvalue,
}
impl From<bool> for ClosureKind {
  fn from(value: bool) -> Self {
    if value {
      Self::Closed
    } else {
      Self::Open
    }
  }
}

#[derive(Clone)]
pub struct Function {
  pub name: String,
  pub arity: Arity,
  pub chunk: Rc<Chunk>,
  pub upvalues: SmallVec<[(u8, ClosureKind); 8]>,
}
impl Default for Function {
  fn default() -> Self {
    Self {
      name: "".into(),
      arity: 0.into(),
      chunk: Chunk::new().into(),
      upvalues: SmallVec::new(),
    }
  }
}

#[derive(Clone)]
pub struct NativeFunction {
  pub name: &'static str,
  pub arity: Arity,
  pub func: fn(args: &[Value]) -> Value,
}
impl NativeFunction {
  pub fn new(name: &'static str, arity: u8, func: fn(args: &[Value]) -> Value) -> Self {
    Self {
      name,
      func,
      arity: arity.into(),
    }
  }
}

#[derive(Clone)]
pub struct Closure {
  pub func: Function,
  pub(crate) upvalues: SmallVec<[Value; 4]>,
}
impl Closure {
  pub fn new(func: Function, upvalues: SmallVec<[Value; 4]>) -> Self {
    Self { func, upvalues }
  }
}
