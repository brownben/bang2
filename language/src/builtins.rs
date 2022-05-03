use crate::{
  value::{calculate_index, NativeFunction, Value},
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
  let to_string = NativeFunction::create("toString", 1, |args| {
    Value::from(match &args[0] {
      Value::String(string) => string.to_string(),
      value => value.to_string(),
    })
  });

  vm.define_global("print", print);
  vm.define_global("type", type_);
  vm.define_global("toString", to_string);
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

  ($name:literal, $function:expr, List,) => {
    NativeFunction::create($name, 1, |args| match &args[0] {
      Value::List(value) => {
        let value = &mut value.borrow_mut();
        Value::from($function(value))
      }
      _ => Value::Null,
    })
  };

  ($name:literal, $function:expr, List, Any,) => {
    NativeFunction::create($name, 2, |args| match &args[0] {
      Value::List(value) => {
        let mut value = value.borrow_mut();
        Value::from($function(&mut value, args[1].clone()))
      }
      _ => Value::Null,
    })
  };

  ($name:literal, $function:expr, List, Any, Returns,) => {
    NativeFunction::create($name, 2, |args| match &args[0] {
      Value::List(value) => {
        let mut value = value.borrow_mut();
        $function(&mut value, args[1].clone());
        args[0].clone()
      }
      _ => Value::Null,
    })
  };

  ($name:literal, $function:expr, List, Number,) => {
    NativeFunction::create($name, 2, |args| match (&args[0], &args[1]) {
      (Value::List(value), Value::Number(number)) => {
        let mut value = value.borrow_mut();
        Value::from($function(&mut value, number.clone()))
      }
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
    $($item_name:literal : ($($item_type:ident,)+) => $item_value:expr,)*
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
      Number "PI":        std::f64::consts::PI,
      Number "E":         std::f64::consts::E,
      Number "INFINITY":  std::f64::INFINITY,
      "floor": (Number,) => f64::floor,
      "ceil":  (Number,) => f64::ceil,
      "round": (Number,) => f64::round,
      "abs":   (Number,) => f64::abs,
      "sqrt":  (Number,) => f64::sqrt,
      "cbrt":  (Number,) => f64::cbrt,
      "sin":   (Number,) => f64::sin,
      "cos":   (Number,) => f64::cos,
      "tan":   (Number,) => f64::tan,
      "asin":  (Number,) => f64::asin,
      "acos":  (Number,) => f64::acos,
      "atan":  (Number,) => f64::atan,
      "sinh":  (Number,) => f64::sinh,
      "cosh":  (Number,) => f64::cosh,
      "tanh":  (Number,) => f64::tanh,
      "asinh": (Number,) => f64::asinh,
      "acosh": (Number,) => f64::acosh,
      "atanh": (Number,) => f64::atanh,
      "isNan": (Number,) => f64::is_nan,
      "exp":   (Number,) => f64::exp,
      "ln":    (Number,) => f64::ln,
      "pow":   (Number, Number,) => f64::powf,
      "log":   (Number, Number,) => f64::log,
      "radiansToDegrees": (Number,) => f64::to_degrees,
      "degreesToRadians": (Number,) => f64::to_radians,
    }),
    "string" => module!(value, {
      "length":      (String,) => str::len,
      "trim":        (String,) => str::trim,
      "trimStart":   (String,) => str::trim_start,
      "trimEnd":     (String,) => str::trim_end,
      "repeat":      (String, Number,) => str::repeat,
      "includes":    (String, String,) => str::contains,
      "startsWith":  (String, String,) => str::starts_with,
      "endsWith":    (String, String,) => str::ends_with,
      "toLowerCase": (String,) => str::to_lowercase,
      "toUpperCase": (String,) => str::to_uppercase,
      "toNumber":    (String,) => |s| str::parse::<f64>(s).unwrap_or(f64::NAN),
    }),
    "list" => module!(value, {
      "length":   (List,) => Vec::len,
      "isEmpty":  (List,) => Vec::is_empty,
      "push":     (List, Any, Returns,) => Vec::push,
      "pop":      (List,) => |list: &mut Vec<_>| list.pop().unwrap_or(Value::Null),
      "includes": (List, Any,) => |list: &mut Vec<_>, value| list.contains(&value),
      "reverse":  (List,) => |list: &mut Vec<_>| list.iter().rev().cloned().collect::<Vec<_>>(),
      "get": (List, Number,) => |list: &mut Vec<_>, index| list.get(calculate_index(index, list.len())).cloned().unwrap_or(Value::Null),
    }),
    _ => None,
  }
}
