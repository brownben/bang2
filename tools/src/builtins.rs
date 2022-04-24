use crate::typechecker::{Type, Typechecker};
use bang_language::ast::expression::LiteralType;

pub fn define_globals(typechecker: &mut Typechecker) {
  let print_arg_existential = typechecker.new_existential();
  let print = Type::Function(
    vec![print_arg_existential.clone()],
    Box::new(print_arg_existential),
  );
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

pub fn get_builtin_module_type(
  typechecker: &mut Typechecker,
  module: &str,
  value: &str,
) -> Option<Type> {
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
    "list" => match value {
      "length" => Some(Type::Function(
        vec![Type::List(Box::new(Type::Any))],
        Box::new(type_!(Number)),
      )),
      "isEmpty" => Some(Type::Function(
        vec![Type::List(Box::new(Type::Any))],
        Box::new(Type::Boolean),
      )),
      "push" => {
        let generic = typechecker.new_existential();
        Some(Type::Function(
          vec![Type::List(Box::new(generic.clone())), generic.clone()],
          Box::new(Type::List(Box::new(generic))),
        ))
      }
      "includes" => {
        let generic = typechecker.new_existential();
        Some(Type::Function(
          vec![Type::List(Box::new(generic.clone())), generic],
          Box::new(Type::Boolean),
        ))
      }
      "pop" => {
        let generic = typechecker.new_existential();
        Some(Type::Function(
          vec![Type::List(Box::new(generic.clone()))],
          Box::new(Type::union(generic, type_!(Null))),
        ))
      }
      "get" => {
        let generic = typechecker.new_existential();
        Some(Type::Function(
          vec![Type::List(Box::new(generic.clone())), type_!(Number)],
          Box::new(Type::union(generic, type_!(Null))),
        ))
      }
      "reverse" => {
        let generic = typechecker.new_existential();
        Some(Type::Function(
          vec![Type::List(Box::new(generic.clone()))],
          Box::new(Type::List(Box::new(generic))),
        ))
      }
      _ => None,
    },

    _ => None,
  }
}
