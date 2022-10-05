use crate::{value::Value, vm::VM};

pub enum ImportValue {
  Constant(Value),
  ModuleNotFound,
  ItemNotFound,
}

pub trait Context {
  fn get_value(&self, module: &str, value: &str) -> ImportValue;
  fn define_globals(&self, vm: &mut VM);
}

pub struct Empty;
impl Context for Empty {
  fn get_value(&self, _: &str, _: &str) -> ImportValue {
    ImportValue::ModuleNotFound
  }
  fn define_globals(&self, _: &mut VM) {}
}
