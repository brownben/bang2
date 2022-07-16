use crate::{
  value::{calculate_index, NativeFunction, Value},
  vm::VM,
};

pub fn define_globals(vm: &mut VM) {
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

macro_rules! count {
  () => (0);
  ( $x:tt $($xs:tt)* ) => (1 + count!($($xs)*));
}

macro_rules! unwrap_type {
  (Number, $args: expr, $do: expr) => {
    unwrap_type!(1, $args, Value::Number(value) => $do(*value).into())
  };
  (String, $args: expr, $do: expr) => {
    unwrap_type!(1, $args, Value::String(value) => $do(value as &str).into())
  };
  (Boolean, $args: expr, $do: expr) => {
    unwrap_type!(1, $args, Value::Boolean(value) => $do(value).into())
  };
  (List, $args: expr, $do: expr) => {
    unwrap_type!(1, $args, Value::List(l) => $do(&mut l.borrow_mut()).into())
  };
  (ListRef, $args: expr, $do: expr) => {
    unwrap_type!(1, $args, Value::List(l) => $do(&l.borrow()).into())
  };
  (Number Number, $args: expr, $do: expr) => {
    unwrap_type!(2, $args, (Value::Number(a), Value::Number(b)) => $do(*a, *b).into())
  };
  (String String, $args: expr, $do: expr) => {
    unwrap_type!(
      2, $args, (Value::String(a), Value::String(b)) => $do(a as &str, b as &str).into()
    )
  };
  (String Usize, $args: expr, $do: expr) => {
    unwrap_type!(
      2, $args, (Value::String(a), Value::Number(b)) => $do(a as &str, *b as usize).into()
    )
  };
  (List Any, $args: expr, $do: expr) => {
    unwrap_type!(
      2, $args, (Value::List(a), b) => $do(&mut a.borrow_mut(), b.clone()).into()
    )
  };
  (ListReturned Any, $args: expr, $do: expr) => {
    unwrap_type!(2, $args, (Value::List(value), b) => {
      $do(&mut value.borrow_mut(), b.clone());
      $args[0].clone()
    })
  };
  (List Number, $args: expr, $do: expr) => {
    unwrap_type!(
      2, $args,  (Value::List(a), Value::Number(b)) => $do(&mut a.borrow_mut(), *b).into()
    )
  };

  (1, $args: expr, $match:pat => $do:expr) => {
    match &$args[0] {
      $match => $do,
      _ => Value::Null,
    }
  };
  (2, $args: expr, $match:pat => $do:expr) => {
    match (&$args[0], &$args[1]) {
      $match => $do,
      _ => Value::Null,
    }
  };
}

macro_rules! module {
  ($key:expr, {
    $(const $value_name:ident = $value:expr;)*
    $(fn $item_name:ident($($type:ident),+) -> $item_value:expr;)*
  }) => {
    match $key {
      $(
        stringify!($value_name) => Some($value.into()),
      )*
      $(
        stringify!($item_name) => Some(
          NativeFunction::new(
            stringify!($item_name),
            count!($($type)+),
            |args| unwrap_type!($($type)+, args, $item_value),
          ).into()
        ),
      )*
      _ => None,
    }
  };
}

pub fn get_builtin_module_value(module: &str, value: &str) -> Option<Value> {
  match module {
    "maths" => module!(value, {
      const PI = std::f64::consts::PI;
      const E = std::f64::consts::E;
      const INFINITY = std::f64::INFINITY;
      fn floor(Number) -> f64::floor;
      fn ceil(Number) -> f64::ceil;
      fn round(Number) -> f64::round;
      fn abs(Number) -> f64::abs;
      fn sqrt(Number) -> f64::sqrt;
      fn cbrt(Number) -> f64::cbrt;
      fn sin(Number) -> f64::sin;
      fn cos(Number) -> f64::cos;
      fn tan(Number) -> f64::tan;
      fn asin(Number) -> f64::asin;
      fn acos(Number) -> f64::acos;
      fn atan(Number) -> f64::atan;
      fn sinh(Number) -> f64::sinh;
      fn cosh(Number) -> f64::cosh;
      fn tanh(Number) -> f64::tanh;
      fn asinh(Number) -> f64::asinh;
      fn acosh(Number) -> f64::acosh;
      fn atanh(Number) -> f64::atanh;
      fn isNan(Number) -> f64::is_nan;
      fn exp(Number) -> f64::exp;
      fn ln(Number) -> f64::ln;
      fn pow(Number, Number) -> f64::powf;
      fn log(Number, Number) -> f64::log;
      fn radiansToDegrees(Number) -> f64::to_degrees;
      fn degreesToRadians(Number) -> f64::to_radians;
    }),
    "string" => module!(value, {
      fn length(String) -> str::len;
      fn trim(String) -> str::trim;
      fn trimStart(String) -> str::trim_start;
      fn trimEnd(String) -> str::trim_end;
      fn repeat(String, Usize) -> str::repeat;
      fn includes(String, String) -> str::contains;
      fn startsWith(String, String) -> str::starts_with;
      fn endsWith(String, String) -> str::ends_with;
      fn toLowerCase(String) -> str::to_lowercase;
      fn toUpperCase(String) -> str::to_uppercase;
      fn toNumber(String) -> |s| str::parse::<f64>(s).unwrap_or(f64::NAN);
    }),
    "fs" => module!(value, {
      fn read(String) -> std::fs::read;
      fn write(String, String) -> std::fs::write;
    }),
    "list" => module!(value, {
      fn length(ListRef) -> Vec::len;
      fn isEmpty(ListRef) -> Vec::is_empty;
      fn push(ListReturned, Any) -> Vec::push;
      fn pop(List) -> Vec::pop;
      fn includes(List, Any) -> |list: &mut Vec<_>, value| list.contains(&value);
      fn reverse(List) -> |l: &mut Vec<_>| l.iter().rev().cloned().collect::<Vec<_>>();
      fn get(List, Number) -> |list: &mut Vec<_>, index| list.get(calculate_index(index, list.len())).cloned();
    }),
    _ => None,
  }
}
