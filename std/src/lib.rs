use bang_interpreter::{
  collections::{HashMap, String},
  context::{Context, ImportValue},
  value::NativeFunction,
  VM,
};
use std::cell::RefCell;

#[macro_use]
mod macros;

mod bytecode;
pub mod modules;

fn construct_module_identifier(module: &str, item: &str) -> String {
  let mut module_identifier = String::new();
  module_identifier.push_str(module);
  module_identifier.push_str("::");
  module_identifier.push_str(item);
  module_identifier
}

#[derive(Default)]
pub struct StdContext {
  import_cache: RefCell<HashMap<String, ImportValue>>,
}
impl Context for StdContext {
  fn get_value(&self, module: &str, item: &str) -> ImportValue {
    let module_identifer = construct_module_identifier(module, item);
    if let Some(value) = self.import_cache.borrow().get(&module_identifer) {
      return value.clone();
    }

    let value = match module {
      "maths" => modules::maths(item),
      "string" => modules::string(item),
      "list" => modules::list(item),
      "set" => modules::set(item),
      "dict" => modules::dict(item),

      #[cfg(feature = "fs")]
      "fs" => modules::fs(item),

      _ => ImportValue::ModuleNotFound,
    };

    self
      .import_cache
      .borrow_mut()
      .insert(module_identifer, value.clone());
    value
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
