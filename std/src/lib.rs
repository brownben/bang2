use bang_interpreter::{Context, NativeFunction, Value, VM};

#[macro_use]
mod macros;

pub mod modules;

pub struct StdContext;
impl Context for StdContext {
  fn get_value(&self, module: &str, value: &str) -> Option<Value> {
    match module {
      "maths" => modules::maths(value),
      "string" => modules::string(value),
      "fs" => modules::fs(value),
      "list" => modules::list(value),
      _ => None,
    }
  }

  fn define_globals(&self, vm: &mut VM) {
    let print = NativeFunction::new("print", 1, |args| {
      match &args[0] {
        Value::String(string) => println!("{}", string),
        value => println!("{}", value),
      };
      args[0].clone()
    });
    let type_ = NativeFunction::new("type", 1, |args| args[0].get_type().into());
    let to_string = NativeFunction::new("toString", 1, |args| match &args[0] {
      value @ Value::String(_) => value.clone(),
      value => value.to_string().into(),
    });

    vm.define_global("print", print.into());
    vm.define_global("type", type_.into());
    vm.define_global("toString", to_string.into());
  }
}
