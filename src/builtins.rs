use crate::{
  value::{NativeFunction, Value},
  vm::VM,
};

pub fn define_globals(vm: &mut VM) {
  let print = NativeFunction::create("print", 1, |args| {
    println!("{}", args[0]);
    Value::Null
  });
  let type_ = NativeFunction::create("type", 1, |args| Value::from(args[0].get_type()));

  vm.define_global("print", print);
  vm.define_global("type", type_);
}
