use crate::{
  value::{NativeFunction, Value},
  vm::VM,
};

fn define_builtin_function(vm: &mut VM, name: &str, arity: u8, func: fn(&[Value]) -> Value) {
  vm.define_global(name, NativeFunction::create(name, arity, func));
}

pub fn define_globals(vm: &mut VM) {
  define_builtin_function(vm, "print", 1, |args| {
    println!("{}", args[0]);
    Value::Null
  });

  define_builtin_function(vm, "type", 1, |args| match args[0] {
    Value::Null => Value::from("null"),
    Value::Number(_) => Value::from("number"),
    Value::String(_) => Value::from("string"),
    Value::Boolean(_) => Value::from("boolean"),
    Value::Function(_) | Value::NativeFunction(_) => Value::from("function"),
  });
}
