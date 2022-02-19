use crate::{
  value::{NativeFunction, Value},
  vm::VM,
};

pub fn define_globals(vm: &mut VM) {
  let print = NativeFunction::create("print", 1, |args| {
    println!("{}", args[0]);
    Value::Null
  });
  let type_ = NativeFunction::create("type", 1, |args| match args[0] {
    Value::Null => Value::from("null"),
    Value::Number(_) => Value::from("number"),
    Value::String(_) => Value::from("string"),
    Value::Boolean(_) => Value::from("boolean"),
    Value::Function(_) | Value::NativeFunction(_) => Value::from("function"),
  });

  vm.define_global("print", print);
  vm.define_global("type", type_);
}
