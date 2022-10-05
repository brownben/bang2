use bang_interpreter::{calculate_index, ImportValue, NativeFunction, Object, Value};
use std::fs;

module!(maths, {
  const PI = std::f64::consts::PI;
  const E = std::f64::consts::E;
  const INFINITY = f64::INFINITY;
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
});

module!(string, {
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
});

module!(fs, {
  fn read(String) -> fs::read_to_string;
  fn write(String, String) -> fs::write;
});

module!(list, {
  fn length(ListRef) -> Vec::len;
  fn isEmpty(ListRef) -> Vec::is_empty;
  fn push(ListReturned, Any) -> Vec::push;
  fn pop(List) -> Vec::pop;
  fn includes(List, Any) -> |list: &mut Vec<_>, value| list.contains(&value);
  fn reverse(List) -> |l: &mut Vec<_>| l.iter().rev().cloned().collect::<Vec<_>>();
  fn get(List, Number) -> |list: &mut Vec<_>, index| list.get(calculate_index(index, list.len())).cloned();
});
