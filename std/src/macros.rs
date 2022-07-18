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
  ($name:ident, {
    $(const $value_name:ident = $value:expr;)*
    $(fn $item_name:ident($($type:ident),+) -> $item_value:expr;)*
  }) => {
    pub fn $name(key: &str) -> Option<Value> {
      match key {
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
    }
  };
}
