use bang_interpreter::{
  collections::{HashMap as BangHashMap, HashSet as BangHashSet},
  context::ImportValue,
  value::{calculate_index, NativeFunction, Object},
  Value,
};
use std::{
  collections::{HashMap, HashSet},
  fs,
};

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
  fn replace(String, String, String) -> str::replace;
  fn replaceOne(String, String, String) -> |a,b,c| str::replacen(a, b, c, 1);
  fn toNumber(String) -> |s| str::parse::<f64>(s).unwrap_or(f64::NAN);
  fn split(String, String) -> |a, b| str::split(a, b).filter(|x| !x.is_empty()).map(Value::from).collect::<Vec<_>>();
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
  fn get(List, Number) -> |l: &mut Vec<_>, i| l.get(calculate_index(i, l.len())).cloned();
  fn toSet(ListRef) -> |l: &Vec<Value>| l.iter().cloned().collect::<BangHashSet<Value>>();
});

module!(set, {
  var fn new() -> BangHashSet::from_iter;
  var fn set() -> BangHashSet::from_iter;
  fn size(SetRef) -> HashSet::len;
  fn isEmpty(SetRef) -> HashSet::is_empty;
  fn insert(Set, Any) -> HashSet::insert;
  fn remove(Set, AnyRef) -> HashSet::remove;
  fn includes(SetRef, AnyRef) -> HashSet::contains;
  fn isDisjoint(Set, Set) -> HashSet::is_disjoint;
  fn isSubset(Set, Set) -> HashSet::is_subset;
  fn isSuperset(Set, Set) -> HashSet::is_superset;
  fn union(SetCloned, Set) -> HashSet::union;
  fn difference(SetCloned, Set) -> HashSet::difference;
  fn intersection(SetCloned, Set) -> HashSet::intersection;
  fn symmetricDifference(SetCloned, Set) -> HashSet::symmetric_difference;
  fn toList(SetRef) -> |s: &BangHashSet<_>| s.iter().cloned().collect::<Vec<_>>();
});

module!(dict, {
  fn new() -> BangHashMap::new;
  fn dict() -> BangHashMap::new;
  fn size(DictRef) -> HashMap::len;
  fn isEmpty(DictRef) -> HashMap::is_empty;
  fn keys(DictRef) -> |d: &BangHashMap<_, _>| d.keys().cloned().collect::<Vec<_>>();
  fn values(DictRef) -> |d: &BangHashMap<_, _>| d.values().cloned().collect::<Vec<_>>();
  fn get(DictRef, Any) -> |dict: &BangHashMap<_, _>, index| dict.get(index).cloned();
});
