use crate::{value::Value, vm::VM};

pub trait Context {
  fn get_value(&self, module: &str, value: &str) -> Option<Value>;
  fn define_globals(&self, vm: &mut VM);
}

pub struct Empty;
impl Context for Empty {
  fn get_value(&self, _: &str, _: &str) -> Option<Value> {
    None
  }
  fn define_globals(&self, _: &mut VM) {}
}
