use super::Value;

#[derive(PartialEq, Eq)]
pub struct Arity {
  count: u8,
  catch_all: bool,
}
impl Arity {
  pub fn new(count: u8, catch_all: bool) -> Self {
    Self { count, catch_all }
  }

  pub fn has_varadic_param(&self) -> bool {
    self.catch_all
  }

  pub fn get_count(&self) -> u8 {
    self.count
  }

  pub fn check_arg_count(&self, provided: u8) -> bool {
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

#[derive(PartialEq, Eq)]
pub struct Function {
  pub name: String,
  pub arity: Arity,
  pub(crate) start: usize,
}

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
