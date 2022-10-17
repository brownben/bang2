use super::Value;
use smallvec::SmallVec;
use smartstring::alias::String;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct Arity {
  count: u8,
  catch_all: bool,
}
impl Arity {
  pub fn new(count: u8, catch_all: bool) -> Self {
    Self { count, catch_all }
  }

  pub fn has_varadic_param(self) -> bool {
    self.catch_all
  }

  pub fn get_count(self) -> u8 {
    self.count
  }

  pub fn check_arg_count(self, provided: u8) -> bool {
    if self.has_varadic_param() {
      provided >= self.count.saturating_sub(1)
    } else {
      self.count == provided
    }
  }
}
impl From<u8> for Arity {
  fn from(count: u8) -> Self {
    Self {
      count,
      catch_all: false,
    }
  }
}

#[derive(Clone, Default, PartialEq, Eq)]
pub struct Function {
  pub name: String,
  pub arity: Arity,
  pub(crate) start: usize,
  pub(crate) upvalues: SmallVec<[(u8, bool); 8]>,
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
  pub fn new_catch_all(name: &'static str, func: fn(args: &[Value]) -> Value) -> Self {
    Self {
      name,
      func,
      arity: Arity::new(1, true),
    }
  }
}

#[derive(Clone)]
pub struct Closure {
  pub func: Function,
  pub(crate) upvalues: SmallVec<[usize; 4]>,
}
impl Closure {
  pub fn new(func: Function, upvalues: SmallVec<[usize; 4]>) -> Self {
    Self { func, upvalues }
  }
}
