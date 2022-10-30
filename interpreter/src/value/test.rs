use super::{Function, NativeFunction, Object, Value};
use std::{
  collections::hash_map::DefaultHasher,
  hash::{Hash, Hasher},
};

#[test]
fn null() {
  let null = Value::NULL;
  assert!(!null.is_number());
  assert!(!null.is_object());
  assert!(!null.is_allocated());
}

#[test]
fn boolean() {
  let true_ = Value::TRUE;
  assert!(!true_.is_number());
  assert!(!true_.is_object());
  assert!(!true_.is_allocated());

  assert_eq!(true_, Value::from(true));

  let false_ = Value::FALSE;
  assert!(!false_.is_number());
  assert!(!false_.is_object());
  assert!(!false_.is_allocated());

  assert_eq!(false_, Value::from(false));
}

#[test]
fn number() {
  for number in [0.0, 1.0, 2.0, 4.0, 8.0, 123.0, -0.0, -2.0, 123.45] {
    let num = Value::from(number);
    assert!(num.is_number());
    assert!(!num.is_object());
    assert!(!num.is_allocated());
    assert_eq!(num.as_number(), number)
  }

  let num = Value::from(f64::NAN);
  assert!(num.is_number());
  assert!(!num.is_object());
  assert!(!num.is_allocated());
  assert!(num.as_number().is_nan());

  let num = Value::from(f64::INFINITY);
  assert!(num.is_number());
  assert!(!num.is_object());
  assert!(!num.is_allocated());
  assert_eq!(num.as_number(), f64::INFINITY);

  let num = Value::from(f64::asin(55.0));
  assert!(num.is_number());
  assert!(!num.is_object());
  assert!(!num.is_allocated());
  assert!(num.as_number().is_nan());
}

#[test]
fn objects() {
  let string = Value::from("hello");
  assert!(string.is_object());
  assert!(!string.is_number());
  assert!(!string.is_allocated());
  assert_eq!(string.as_object(), Object::String("hello".into()).into());
}

#[test]
fn is_falsy() {
  assert_eq!(Value::TRUE.is_falsy(), false);
  assert_eq!(Value::FALSE.is_falsy(), true);
  assert_eq!(Value::NULL.is_falsy(), true);

  assert_eq!(Value::from(0).is_falsy(), true);
  assert_eq!(Value::from(-0).is_falsy(), true);
  assert_eq!(Value::from(0.01).is_falsy(), false);
  assert_eq!(Value::from(123).is_falsy(), false);

  assert_eq!(Value::from("").is_falsy(), true);
  assert_eq!(Value::from("hello").is_falsy(), false);

  let function = Function {
    name: "hello".into(),
    ..Default::default()
  };
  assert_eq!(Value::from(function).is_falsy(), false);

  assert_eq!(Value::from(Vec::new()).is_falsy(), true);
  assert_eq!(Value::from(vec![123.into()]).is_falsy(), false);
}

#[test]
fn get_type() {
  assert_eq!(Value::TRUE.get_type(), "boolean");
  assert_eq!(Value::FALSE.get_type(), "boolean");
  assert_eq!(Value::NULL.get_type(), "null");

  assert_eq!(Value::from(0).get_type(), "number");
  assert_eq!(Value::from(123).get_type(), "number");

  assert_eq!(Value::from("").get_type(), "string");
  assert_eq!(Value::from("hello").get_type(), "string");

  let function = Function {
    name: "hello".into(),
    ..Default::default()
  };
  assert_eq!(Value::from(function).get_type(), "function");

  let native_function = NativeFunction {
    name: "native",
    arity: 0.into(),
    func: |_| Value::NULL,
  };
  assert_eq!(Value::from(native_function).get_type(), "function");

  assert_eq!(Value::from(Vec::new()).get_type(), "list");
  assert_eq!(Value::from(vec![123.into()]).get_type(), "list");
}

#[test]
fn displays_correctly() {
  assert_eq!(Value::from("hello").to_string(), "hello");
  assert_eq!(Value::from(true).to_string(), "true");
  assert_eq!(Value::from(false).to_string(), "false");
  assert_eq!(Value::from(12.345).to_string(), "12.345");
  assert_eq!(Value::from(Vec::new()).to_string(), "[]");
  assert_eq!(
    Value::from(vec!["hello".into(), 7.into()]).to_string(),
    "['hello', 7]"
  );
  assert_eq!(
    Value::from(vec!["hello".into(), vec![].into()]).to_string(),
    "['hello', []]"
  );
  assert_eq!(
    Value::from(Function {
      name: "hello".into(),
      ..Default::default()
    })
    .to_string(),
    "<function hello>"
  );

  assert_eq!(
    Value::from(NativeFunction {
      name: "native",
      arity: 0.into(),
      func: |_| Value::NULL
    })
    .to_string(),
    "<function native>"
  );
}

#[test]
fn equality() {
  let function = Value::from(Function {
    name: "hello".into(),
    ..Default::default()
  });
  let native_base = NativeFunction {
    name: "native",
    arity: 0.into(),
    func: |_| Value::NULL,
  };
  let number = Value::from(0);

  assert_eq!(Value::TRUE, Value::from(true));
  assert_ne!(Value::FALSE, Value::TRUE);
  assert_eq!(Value::NULL, Value::from(()));
  assert_eq!(Value::from(4usize), Value::from(4.0));
  assert_ne!(Value::from(3.6), Value::from(3.61));
  assert_eq!(Value::from('a'), Value::from("a".to_string()));
  assert_eq!(number, number.clone());
  assert_eq!(function, function.clone());
  assert_ne!(
    Value::from(native_base.clone()),
    Value::from(native_base.clone())
  );
  assert_ne!(Value::from(native_base), function);
  assert_eq!(Value::from(vec![]), Value::from(vec![]));
  assert_ne!(
    Value::from(vec!["hello".into(), 7.into()]),
    Value::from(vec![])
  );

  let result_ok: Result<f64, bool> = Ok(3.5);
  let result_error: Result<f64, bool> = Err(false);
  assert_eq!(Value::from(result_ok), Value::from(3.5));
  assert_eq!(Value::from(result_error), Value::NULL);
  assert_ne!(Value::from(result_error), Value::FALSE);
}

macro_rules! assert_hash_eq {
  ($a:expr, $b:expr) => {
    let hasher_a = &mut DefaultHasher::new();
    $a.hash(hasher_a);
    let a = hasher_a.finish();

    let hasher_b = &mut DefaultHasher::new();
    $b.hash(hasher_b);
    let b = hasher_b.finish();

    assert_eq!(a, b);
  };
}

macro_rules! assert_hash_ne {
  ($a:expr, $b:expr) => {
    let hasher_a = &mut DefaultHasher::new();
    $a.hash(hasher_a);
    let a = hasher_a.finish();

    let hasher_b = &mut DefaultHasher::new();
    $b.hash(hasher_b);
    let b = hasher_b.finish();

    assert_ne!(a, b);
  };
}

#[test]
fn hash() {
  assert_hash_eq!(Value::from(()), Value::NULL);
  assert_hash_ne!(Value::from(0), Value::NULL);
  assert_hash_ne!(Value::from(0), Value::FALSE);

  assert_hash_eq!(Value::TRUE, Value::from(true));
  assert_hash_eq!(Value::FALSE, Value::from(false));
  assert_hash_ne!(Value::TRUE, Value::FALSE);

  let function = Value::from(Function {
    name: "hello".into(),
    ..Default::default()
  });
  let native = Value::from(NativeFunction {
    name: "native",
    arity: 0.into(),
    func: |_| Value::NULL,
  });
  assert_hash_eq!(function, function.clone());
  assert_hash_ne!(function, native);

  assert_hash_eq!(Value::from("Hello"), Value::from("Hello"));
  assert_hash_ne!(Value::from("Hello"), Value::from("Hell0"));
  assert_hash_ne!(Value::from("Hello"), Value::from("hello"));

  let list = Value::from(vec![1.into(), 2.into(), 3.into()]);
  let list_b = Value::from(vec![1.into(), 2.into(), 3.into()]);
  assert_hash_eq!(list, list.clone());
  assert_hash_ne!(list, list_b);
}

#[test]
fn allocate() {
  let string = Value::from("hello");
  assert!(string.is_object());
  assert!(!string.is_number());
  assert!(!string.is_allocated());
  assert_eq!(string.as_object(), Object::String("hello".into()).into());

  let string = string.allocate();
  assert!(string.is_allocated());
  assert!(!string.is_object());
  assert!(!string.is_number());
  assert_eq!(
    *string.as_allocated().borrow(),
    Value::from(Object::String("hello".into()))
  );
  assert_eq!(string.as_allocated().clone(), string.as_allocated().clone(),);
}
