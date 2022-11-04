use bang_interpreter::{
  context::{Context, ImportValue},
  value::NativeFunction,
  VM,
};

#[macro_use]
mod macros;

pub mod modules;

pub struct StdContext;
impl Context for StdContext {
  fn get_value(&self, module: &str, value: &str) -> ImportValue {
    match module {
      "maths" => modules::maths(value),
      "string" => modules::string(value),
      "fs" => modules::fs(value),
      "list" => modules::list(value),
      "set" => modules::set(value),
      "dict" => modules::dict(value),
      _ => ImportValue::ModuleNotFound,
    }
  }

  fn define_globals(&self, vm: &mut VM) {
    let print = NativeFunction::new("print", 1, |args| {
      println!("{}", &args[0]);
      args[0].clone()
    });
    let type_ = NativeFunction::new("type", 1, |args| args[0].get_type().into());
    let to_string = NativeFunction::new("toString", 1, |args| args[0].to_string().into());

    vm.define_global("print", print.into());
    vm.define_global("type", type_.into());
    vm.define_global("toString", to_string.into());
  }
}
