use crate::{
  value::{NativeFunction, Value},
  vm::VM,
};

pub fn define_globals(vm: &mut VM) {
  let print = NativeFunction::create("print", 1, |args| {
    match &args[0] {
      Value::String(string) => println!("{}", string),
      value => println!("{}", value),
    }
    args[0].clone()
  });
  let type_ = NativeFunction::create("type", 1, |args| Value::from(args[0].get_type()));

  vm.define_global("print", print);
  vm.define_global("type", type_);
}

macro_rules! function_wrapper {
  ($name:literal, $function:expr, Number,) => {
    NativeFunction::create($name, 1, |args| match args[0] {
      Value::Number(value) => Value::from($function(value)),
      _ => Value::Null,
    })
  };

  ($name:literal, $function:expr, Number, Number,) => {
    NativeFunction::create($name, 2, |args| match (&args[0], &args[1]) {
      (Value::Number(first), Value::Number(second)) => Value::from($function(*first, *second)),
      _ => Value::Null,
    })
  };

  ($name:literal, $function:expr, String,) => {
    NativeFunction::create($name, 1, |args| match &args[0] {
      Value::String(value) => Value::from($function(&value)),
      _ => Value::Null,
    })
  };

  ($name:literal, $function:expr, String, String,) => {
    NativeFunction::create($name, 2, |args| match (&args[0], &args[1]) {
      (Value::String(first), Value::String(second)) => {
        Value::from($function(&first as &str, &second as &str))
      }
      _ => Value::Null,
    })
  };

  ($name:literal, $function:expr, String, Number,) => {
    NativeFunction::create($name, 2, |args| match (&args[0], &args[1]) {
      (Value::String(first), Value::Number(second)) => {
        Value::from($function(&first, *second as usize))
      }
      _ => Value::Null,
    })
  };
}

macro_rules! module {
  ($key:expr, {
    $($type:ident $value_name:literal :  $value:expr,)*
    $($item_name:literal : $item_value:expr, ($($item_type:ident,)+),)*
  }) => {
    match $key {
      $(
        $value_name => Some(Value::from($value)),
      )*
      $(
        $item_name => Some(function_wrapper!($item_name, $item_value, $($item_type,)+)),
      )*
      _ => None,
    }
  };
}

pub fn get_builtin_module_value(module: &str, value: &str) -> Option<Value> {
  match module {
    "maths" => module!(value, {
      Number "PI":       std::f64::consts::PI,
      Number "E":        std::f64::consts::E,
      Number "INFINITY": std::f64::INFINITY,
      "floor": f64::floor,  (Number,),
      "ceil":  f64::ceil,   (Number,),
      "round": f64::round,  (Number,),
      "abs":   f64::abs,    (Number,),
      "sqrt":  f64::sqrt,   (Number,),
      "cbrt":  f64::cbrt,   (Number,),
      "exp":   f64::exp,    (Number,),
      "pow":   f64::powf,   (Number, Number,),
      "log":   f64::log,    (Number, Number,),
      "ln":    f64::ln,     (Number,),
      "sin":   f64::sin,    (Number,),
      "cos":   f64::cos,    (Number,),
      "tan":   f64::tan,    (Number,),
      "asin":  f64::asin,   (Number,),
      "acos":  f64::acos,   (Number,),
      "atan":  f64::atan,   (Number,),
      "sinh":  f64::sinh,   (Number,),
      "cosh":  f64::cosh,   (Number,),
      "tanh":  f64::tanh,   (Number,),
      "asinh": f64::asinh,  (Number,),
      "acosh": f64::acosh,  (Number,),
      "atanh": f64::atanh,  (Number,),
      "isNan": f64::is_nan, (Number,),
      "radiansToDegrees": f64::to_degrees, (Number,),
      "degreesToRadians": f64::to_radians, (Number,),
    }),
    "string" => module!(value, {
      "length":      str::len,          (String,),
      "trim":        str::trim,         (String,),
      "trimStart":   str::trim_start,   (String,),
      "trimEnd":     str::trim_end,     (String,),
      "repeat":      str::repeat,       (String, Number,),
      "includes":    str::contains,     (String, String,),
      "startsWith":  str::starts_with,  (String, String,),
      "endsWith":    str::ends_with,    (String, String,),
      "toLowerCase": str::to_lowercase, (String,),
      "toUpperCase": str::to_uppercase, (String,),
      "toNumber":  |s| str::parse::<f64>(s).unwrap_or(f64::NAN), (String,),
    }),
    _ => None,
  }
}
