macro_rules! count {
  () => (0);
  ( $x:tt $($xs:tt)* ) => (1 + count!($($xs)*));
}

macro_rules! unwrap_type {
  (, $args: expr, $do: expr) => {{
    $do().into()
  }};
  (Number, $args: expr, $do: expr) => {{
    if $args[0].is_number() {
      return $do($args[0].as_number()).into();
    }
    Value::NULL
  }};
  (String, $args: expr, $do: expr) => {{
    if $args[0].is_object() {
      if let Object::String(value) = $args[0].as_object() {
        return $do(value as &str).into();
      }
    }
    Value::NULL
  }};
  (List, $args: expr, $do: expr) => {{
    if $args[0].is_object() {
      if let Object::List(value) = $args[0].as_object() {
        return $do(&mut value.borrow_mut()).into();
      }
    }
    Value::NULL
  }};
  (ListRef, $args: expr, $do: expr) => {{
    if $args[0].is_object() {
      if let Object::List(value) = $args[0].as_object() {
        return $do(&value.borrow()).into();
      }
    }
    Value::NULL
  }};
  (Number Number, $args: expr, $do: expr) => {{
    if $args[0].is_number() && $args[1].is_number() {
      return $do($args[0].as_number(), $args[1].as_number()).into();
    }
    Value::NULL
  }};
  (String String, $args: expr, $do: expr) => {{
    if $args[0].is_object() && $args[1].is_object() {
      if let Object::String(a) = $args[0].as_object() {
        if let Object::String(b) = $args[1].as_object() {
          return $do(a as &str, b as &str).into();
        }
      }
    }
    Value::NULL
  }};
  (String String String, $args: expr, $do: expr) => {{
    if $args[0].is_object() && $args[1].is_object() && $args[2].is_object() {
      if let Object::String(a) = $args[0].as_object() {
        if let Object::String(b) = $args[1].as_object() {
          if let Object::String(c) = $args[2].as_object() {
            return $do(a as &str, b as &str, c as &str).into();
          }
        }
      }
    }
    Value::NULL
  }};
  (String Usize, $args: expr, $do: expr) => {{
    if $args[0].is_object() && $args[1].is_number() {
      if let Object::String(value) = $args[0].as_object() {
        return $do(value as &str, $args[1].as_number() as usize).into();
      }
    }
    Value::NULL
  }};
  (List Any, $args: expr, $do: expr) => {{
    if $args[0].is_object() {
      if let Object::List(value) = $args[0].as_object() {
        return $do(&mut value.borrow_mut(), $args[1].clone()).into();
      }
    }
    Value::NULL
  }};
  (ListReturned Any, $args: expr, $do: expr) => {{
    if $args[0].is_object() {
      if let Object::List(value) = $args[0].as_object() {
        $do(&mut value.borrow_mut(), $args[1].clone());
        return $args[0].clone();
      }
    }
    Value::NULL
  }};
  (List Number, $args: expr, $do: expr) => {{
    if $args[0].is_object() && $args[1].is_number() {
      if let Object::List(value) = $args[0].as_object() {
        return $do(&mut value.borrow_mut(), $args[1].as_number()).into();
      }
    }
    Value::NULL
  }};
  (SetRef, $args: expr, $do: expr) => {{
    if $args[0].is_object() {
      if let Object::Set(value) = $args[0].as_object() {
        return $do(&value.borrow()).into();
      }
    }
    Value::NULL
  }};
  (Set Any, $args: expr, $do: expr) => {{
    if $args[0].is_object() {
      if let Object::Set(value) = $args[0].as_object() {
        return $do(&mut value.borrow_mut(), $args[1].clone()).into();
      }
    }
    Value::NULL
  }};
  (Set AnyRef, $args: expr, $do: expr) => {{
    if $args[0].is_object() {
      if let Object::Set(value) = $args[0].as_object() {
        return $do(&mut value.borrow_mut(), &$args[1].clone()).into();
      }
    }
    Value::NULL
  }};
  (SetRef AnyRef, $args: expr, $do: expr) => {{
    if $args[0].is_object() {
      if let Object::Set(value) = $args[0].as_object() {
        return $do(&value.borrow(), &$args[1].clone()).into();
      }
    }
    Value::NULL
  }};
  (Set Set, $args: expr, $do: expr) => {{
    if $args[0].is_object() && $args[1].is_object() {
      if let Object::Set(a) = $args[0].as_object() {
        if let Object::Set(b) = $args[1].as_object() {
          return $do(&a.borrow(), &b.borrow()).into();
        }
      }
    }
    Value::NULL
  }};
  (SetCloned Set, $args: expr, $do: expr) => {{
    if $args[0].is_object() && $args[1].is_object() {
      if let Object::Set(a) = $args[0].as_object() {
        if let Object::Set(b) = $args[1].as_object() {
          return $do(&a.borrow(), &b.borrow())
            .cloned()
            .collect::<BangHashSet<Value>>()
            .into();
        }
      }
    }
    Value::NULL
  }};
  (Varadic, $args: expr, $do: expr) => {{
    if $args[0].is_object() {
      if let Object::List(a) = $args[0].as_object() {
        return $do(a.borrow().iter().cloned()).into();
      }
    }
    Value::NULL
  }};
  (DictRef, $args: expr, $do: expr) => {{
    if $args[0].is_object() {
      if let Object::Dict(value) = $args[0].as_object() {
        return $do(&value.borrow()).into();
      }
    }
    Value::NULL
  }};
  (DictRef Any, $args: expr, $do: expr) => {{
    if $args[0].is_object() {
      if let Object::Dict(value) = $args[0].as_object() {
        return $do(&value.borrow(), &$args[1]).into();
      }
    }
    Value::NULL
  }};
}

macro_rules! module {
  ($name:ident, {
    $(const $value_name:ident = $value:expr;)*
    $(var fn $var_item_name:ident() -> $var_item_value:expr;)*
    $(fn $item_name:ident($($type:ident),*) -> $item_value:expr;)*
    $(bytecode fn $bytecode_item_name:ident($($by_type:ident),*) -> $bytecode_item_value:expr;)*
  }) => {
    pub fn $name(key: &str) -> ImportValue {
      #![allow(unused_variables)]

      match key {
        $(
          stringify!($value_name) => ImportValue::Constant($value.into()),
        )*
        $(
          stringify!($item_name) => ImportValue::Constant(
            NativeFunction::new(
              concat!(stringify!($name), "::", stringify!($item_name)),
              count!($($type)*),
              |args| unwrap_type!($($type)*, args, $item_value),
            ).into()
          ),
        )*
        $(
          stringify!($var_item_name) => ImportValue::Constant(
            NativeFunction::new_catch_all(
              concat!(stringify!($name), "::", stringify!($var_item_name)),
              |args| unwrap_type!(Varadic, args, $var_item_value),
            ).into()
          ),
        )*
        $(
          stringify!($bytecode_item_name) => {
            let mut function = Function::default();
            function.name = concat!(
              stringify!($name),
              "::",
              stringify!($bytecode_item_name)
            ).into();
            function.arity = count!($($by_type)*).into();

            ImportValue::Bytecode($bytecode_item_value, function)
          }
        )*
        _ => ImportValue::ItemNotFound,
      }
    }
  };
}

macro_rules! bytecode {
  ($builder:expr, long $value:expr) => {{
    $builder.emit_long_value($value);
  }};
  ($builder:expr, const $value:expr) => {{
    $builder.emit_constant($value.into());
  }};
  ($builder:expr, $value:literal) => {{
    $builder.emit_value($value);
  }};
  ($builder:expr, $op:ident) => {{
    $builder.emit_opcode(OpCode::$op);
  }};

  ($($ty:tt $($item:expr)?,)+) => {{
    |mut builder| {
      $({ bytecode!(builder, $ty $($item)?) })+;
      builder.finish()
    }
  }};
}
