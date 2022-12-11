use super::{HashMap, Type, Typechecker};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImportValue {
  Value(Type),
  ModuleNotFound,
  ItemNotFound,
}

macro_rules! register_globals {
  ($typechecker:expr, { $($name:ident: $text:literal,)* }) => {
    $(
      let annotation = $typechecker.type_from_annotation(
        &bang_syntax::parse_type($text).expect("to be valid syntax"),
        &mut HashMap::default(),
      ).expect("annotation to be valid");

      $typechecker.scope.insert(stringify!($name), annotation);
    )*

    $typechecker.scope.begin_scope();
  };
}

macro_rules! module {
  ($item:expr, $typechecker:expr, { $($name:ident: $text:literal,)* }) => {
    match $item {
      $(
        stringify!($name) => {
          ImportValue::Value(
            $typechecker.type_from_annotation(
              &bang_syntax::parse_type($text).expect("to be valid syntax"),
              &mut HashMap::default(),
            ).expect("annotation to be valid")
          )
        }
      )*
      _ => ImportValue::ItemNotFound,
    }
  };
}

impl Typechecker<'_> {
  pub fn get_module_item(&mut self, module: &str, item: &str) -> ImportValue {
    match module {
      "maths" => module!(item, self, {
        PI: "number",
        E: "number",
        INFINITY: "number",
        floor: "(number) -> number",
        ceil: "(number) -> number",
        round: "(number) -> number",
        abs: "(number) -> number",
        sqrt: "(number) -> number",
        cbrt: "(number) -> number",
        sin: "(number) -> number",
        cos: "(number) -> number",
        tan: "(number) -> number",
        asin: "(number) -> number",
        acos: "(number) -> number",
        atan: "(number) -> number",
        sinh: "(number) -> number",
        cosh: "(number) -> number",
        tanh: "(number) -> number",
        asinh: "(number) -> number",
        acosh: "(number) -> number",
        atanh: "(number) -> number",
        isNan: "(number) -> boolean",
        exp: "(number) -> number",
        ln: "(number) -> number",
        pow: "(number, number) -> number",
        log: "(number, number) -> number",
        radiansToDegrees: "(number) -> number",
        degreesToRadians: "(number) -> number",
      }),
      "string" => module!(item, self, {
        NEW_LINE: "string",
        TAB: "string",
        CARRIAGE_RETURN: "string",
        length: "(string) -> number",
        trim: "(string) -> string",
        trimStart: "(string) -> string",
        trimEnd: "(string) -> string",
        repeat: "(string, number) -> string",
        includes: "(string, string) -> boolean",
        startsWith: "(string, string) -> boolean",
        endsWith: "(string, string) -> boolean",
        toUpperCase: "(string) -> string",
        toLowerCase: "(string) -> string",
        replace: "(string, string, string) -> string",
        replaceOne: "(string, string, string) -> string",
        toNumber: "(string) -> number?",
        split: "(string, string) -> string[]",
      }),
      "fs" => module!(item, self, {
        read: "(string) -> string?",
        write: "(string, string) -> boolean",
      }),
      "list" => module!(item, self, {
        length: "<T>(T[]) -> number",
        isEmpty: "<T>(T[]) -> boolean",
        push: "<T>(T[], T) -> T[]",
        pop: "<T>(T[]) -> T?",
        includes: "<T>(T[], T) -> boolean",
        reverse: "<T>(T[]) -> T[]",
        get: "<T>(T[], number) -> T?",
        toSet: "<T>(T[]) -> set(T)",
        any: "<T>(T[], (T) -> boolean) -> boolean",
        all: "<T>(T[], (T) -> boolean) -> boolean",
        map: "<S, T>(T[], (T) -> S) -> S[]",
        filter: "<T>(T[], (T) -> boolean) -> T[]",
        reduce: "<S, T>(T[], S, (T) -> S) -> S",
        find: "<T>(T[], (T) -> boolean) -> T?",
        forEach: "<T>(T[], (T) -> any) -> null",
        max: "((string[]) -> string) | ((number[]) -> number)",
        min: "((string[]) -> string) | ((number[]) -> number)",
      }),
      "set" => module!(item, self, {
        new: "<T>(..T) -> set(T)",
        set: "<T>(..T) -> set(T)",
        size: "<T>(set(T)) -> number",
        isEmpty: "<T>(set(T)) -> boolean",
        insert: "<T>(set(T), T) -> boolean",
        remove: "<T>(set(T), T) -> boolean",
        includes: "<T>(set(T), T) -> boolean",
        isDisjoint: "<T>(set(T), set(T)) -> boolean",
        isSuperset: "<T>(set(T), set(T)) -> boolean",
        isSubset: "<T>(set(T), set(T)) -> boolean",
        union: "<T>(set(T), set(T)) -> set(T)",
        difference: "<T>(set(T), set(T)) -> set(T)",
        intersection: "<T>(set(T), set(T)) -> set(T)",
        symmetricDifference: "<T>(set(T), set(T)) -> set(T)",
      }),
      "dict" => module!(item, self, {
        new: "<S, T>() -> dict(S, T)",
        dict: "<S, T>() -> dict(S, T)",
        size: "<S, T>(dict(S, T)) -> number",
        isEmpty: "<S, T>(dict(S, T)) -> bool",
        keys: "<S, T>(dict(S, T)) -> S[]",
        values: "<S, T>(dict(S, T)) -> S[]",
        get: "<S, T>(dict(S, T), S) -> T?",
      }),
      _ => ImportValue::ModuleNotFound,
    }
  }
}
