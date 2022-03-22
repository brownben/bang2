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

macro_rules! create_module {
  ($name:ident, { $($item_name:literal : $item_value:expr,)* }) => {
    mod $name {
      use crate::value::{NativeFunction, Value};

      pub fn get_value(value: &str) -> Option<Value> {
        match value {
          $(
            $item_name => Some($item_value),
          )*
          _ => None,
        }
      }
    }
  };
}
macro_rules! std_wrapper {
  ($name:literal, $std_function:expr, Number) => {
    NativeFunction::create($name, 1, |args| match args[0] {
      Value::Number(value) => Value::from($std_function(value)),
      _ => Value::Null,
    })
  };

  ($name:literal, $std_function:expr, Number, Number) => {
    NativeFunction::create($name, 2, |args| match (&args[0], &args[1]) {
      (Value::Number(first), Value::Number(second)) => Value::from($std_function(*first, *second)),
      _ => Value::Null,
    })
  };

  ($name:literal, $std_function:expr, String) => {
    NativeFunction::create($name, 1, |args| match &args[0] {
      Value::String(value) => Value::from($std_function(&value)),
      _ => Value::Null,
    })
  };

  ($name:literal, $std_function:expr, String, String) => {
    NativeFunction::create($name, 2, |args| match (&args[0], &args[1]) {
      (Value::String(first), Value::String(second)) => {
        Value::from($std_function(&first as &str, &second as &str))
      }
      _ => Value::Null,
    })
  };

  ($name:literal, $std_function:expr, String, Number) => {
    NativeFunction::create($name, 2, |args| match (&args[0], &args[1]) {
      (Value::String(first), Value::Number(second)) => {
        Value::from($std_function(&first, *second as usize))
      }
      _ => Value::Null,
    })
  };
}

pub fn get_builtin_module_value(module: &str, value: &str) -> Option<Value> {
  match module {
    "maths" => maths::get_value(value),
    "string" => string::get_value(value),
    _ => None,
  }
}

create_module!(maths, {
  "PI": Value::from(std::f64::consts::PI),
  "E": Value::from(std::f64::consts::E),
  "INFINITY": Value::from(std::f64::INFINITY),
  "floor": std_wrapper!("floor", f64::floor, Number),
  "ceil": std_wrapper!("ceil", f64::ceil, Number),
  "round": std_wrapper!("round", f64::round, Number),
  "abs": std_wrapper!("abs", f64::abs, Number),
  "sqrt": std_wrapper!("sqrt", f64::sqrt, Number),
  "cbrt": std_wrapper!("cbrt", f64::cbrt, Number),
  "exp": std_wrapper!("exp", f64::exp, Number),
  "pow": std_wrapper!("pow", f64::powf, Number, Number),
  "log": std_wrapper!("log", f64::log, Number, Number),
  "ln": std_wrapper!("ln", f64::ln, Number),
  "sin": std_wrapper!("sin", f64::sin, Number),
  "cos": std_wrapper!("cos", f64::cos, Number),
  "tan": std_wrapper!("tan", f64::tan, Number),
  "asin": std_wrapper!("asin", f64::asin, Number),
  "acos": std_wrapper!("acos", f64::acos, Number),
  "atan": std_wrapper!("atan", f64::atan, Number),
  "sinh": std_wrapper!("sinh", f64::sinh, Number),
  "cosh": std_wrapper!("cosh", f64::cosh, Number),
  "tanh": std_wrapper!("tanh", f64::tanh, Number),
  "asinh": std_wrapper!("asinh", f64::asinh, Number),
  "acosh": std_wrapper!("acosh", f64::acosh, Number),
  "atanh": std_wrapper!("atanh", f64::atanh, Number),
  "isNan": std_wrapper!("isNan", f64::is_nan, Number),
  "radiansToDegrees": std_wrapper!("radiansToDegrees", f64::to_degrees, Number),
  "degreesToRadians": std_wrapper!("degreesToRadians", f64::to_radians, Number),
});

create_module!(string, {
  "length": std_wrapper!("length", str::len, String),
  "trim": std_wrapper!("trim", str::trim, String),
  "trimStart": std_wrapper!("trimStart", str::trim_start, String),
  "trimEnd": std_wrapper!("trimEnd", str::trim_end, String),
  "repeat": std_wrapper!("repeat", str::repeat, String, Number),
  "includes": std_wrapper!("includes", str::contains, String, String),
  "startsWith": std_wrapper!("startsWith", str::starts_with, String, String),
  "endsWith": std_wrapper!("endsWith", str::ends_with, String, String),
  "toLowerCase": std_wrapper!("toLowerCase", str::to_lowercase, String),
  "toUpperCase": std_wrapper!("toUpperCase", str::to_uppercase, String),
  "toNumber": std_wrapper!("toNumber", |str| str::parse::<f64>(str).unwrap_or(f64::NAN), String),
});
