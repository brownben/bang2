use crate::value::{NativeFunction, Value};
use crate::vm::VM;

use std::rc::Rc;

fn define_builtin_function(vm: &mut VM, name: &str, arity: u8, func: fn(&[Value]) -> Value) {
  vm.globals.insert(
    Rc::from(name.to_string()),
    NativeFunction::create(name, arity, func),
  );
}

pub fn define_globals(vm: &mut VM) {
  define_builtin_function(vm, "print", 1, |args| {
    println!("{}", args[0]);
    Value::Null
  });

  define_builtin_function(vm, "type", 1, |args| match args[0] {
    Value::Number(_) => Value::from("number"),
    Value::String(_) => Value::from("string"),
    Value::Boolean(_) => Value::from("boolean"),
    Value::Null => Value::from("null"),
    Value::Function(_) | Value::NativeFunction(_) => Value::from("function"),
  });
}
