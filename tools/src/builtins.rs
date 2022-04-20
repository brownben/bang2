use crate::typechecker::{Type, Typechecker};
use bang_language::ast::expression::LiteralType;

pub fn define_globals(typechecker: &mut Typechecker) {
  let print = Type::Function(vec![Type::Any], Box::new(Type::Literal(LiteralType::Null)));
  let type_ = Type::Function(
    vec![Type::Any],
    Box::new(Type::Literal(LiteralType::String)),
  );

  typechecker.define("print", &print);
  typechecker.define("type", &type_);
}

macro_rules! type_ {
  (Boolean) => {
    Type::Boolean
  };
  (NumberOrNull) => {
    Type::Union(Box::new(type_!(Number)), Box::new(type_!(Null)))
  };
  ($type:ident) => {
    Type::Literal(LiteralType::$type)
  };
}

macro_rules! module {
  ($key:expr, {
    $($type:ident $value_name:literal,)*
    $($item_name:literal : ($($item_type:ident,)+) -> $returns:ident,)*
  }) => {
    match $key {
      $(
        $value_name => Some(type_!($type)),
      )*
      $(
        $item_name => Some(
          Type::Function(
            vec![
              $(
                type_!($item_type),
              )+
            ],
            Box::new(type_!($returns))
          )
        ),
      )*
      _ => None,
    }
  };
}

pub fn get_builtin_module_type(module: &str, value: &str) -> Option<Type> {
  match module {
    "maths" => module!(value, {
      Number "PI",
      Number "E",
      Number "INFINITY",
      "floor": (Number,) -> Number,
      "ceil":  (Number,) -> Number,
      "round": (Number,) -> Number,
      "abs":   (Number,) -> Number,
      "sqrt":  (Number,) -> Number,
      "cbrt":  (Number,) -> Number,
      "exp":   (Number,) -> Number,
      "pow":   (Number, Number,) -> Number,
      "log":   (Number, Number,) -> Number,
      "ln":    (Number,) -> Number,
      "sin":   (Number,) -> Number,
      "cos":   (Number,) -> Number,
      "tan":   (Number,) -> Number,
      "asin":  (Number,) -> Number,
      "acos":  (Number,) -> Number,
      "atan":  (Number,) -> Number,
      "sinh":  (Number,) -> Number,
      "cosh":  (Number,) -> Number,
      "tanh":  (Number,) -> Number,
      "asinh": (Number,) -> Number,
      "acosh": (Number,) -> Number,
      "atanh": (Number,) -> Number,
      "isNan": (Number,) -> Boolean,
      "radiansToDegrees": (Number,) -> Number,
      "degreesToRadians": (Number,) -> Number,
    }),
    "string" => module!(value, {
      "length":      (String,) -> Number,
      "trim":        (String,) -> String,
      "trimStart":   (String,) -> String,
      "trimEnd":     (String,) -> String,
      "repeat":      (String, Number,) -> String,
      "includes":    (String, String,) -> Boolean,
      "startsWith":  (String, String,) -> Boolean,
      "endsWith":    (String, String,) -> Boolean,
      "toLowerCase": (String,) -> String,
      "toUpperCase": (String,) -> String,
      "toNumber":    (String,) -> NumberOrNull,
    }),
    _ => None,
  }
}
