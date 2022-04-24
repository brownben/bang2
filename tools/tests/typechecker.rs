use bang_language::parse;
use bang_tools::typecheck;

macro_rules! assert_correct {
  ($source:expr) => {
    let ast = parse($source).unwrap();
    let result = typecheck($source, &ast);

    assert!(result.is_empty(), "{result:?}");
  };
}

macro_rules! assert_fails {
  ($source:expr) => {
    let ast = parse($source).unwrap();
    let result = typecheck($source, &ast);

    assert!(result.len() > 0, "Test Passes");
  };
}

#[test]
fn literals() {
  assert_correct!("let a: string = 'Hello, World!'");
  assert_correct!("let a: number = 42");
  assert_correct!("let a: boolean = true");
  assert_correct!("let a: boolean = false");
  assert_correct!("let a: null = null");
}

#[test]
fn declarations() {
  assert_correct!("let a = 42\nlet b: number = a\n");
  assert_correct!("let a = true\nlet b: boolean = a\n");
  assert_correct!("let a = null\nlet b: null = a\n");
  assert_correct!("let a\nlet b: null = a\n");
}

#[test]
fn typed_declarations() {
  assert_correct!("let a: number = 42\nlet b: number = a\n");
  assert_correct!("let a: boolean = true\nlet b: boolean = a\n");
  assert_correct!("let a: null = null\nlet b: null = a\n");
  assert_correct!("let a: null\nlet b: null = a\n");
  assert_correct!("let a: null | null\nlet b: null = a\n");
  assert_correct!("let a: null?\nlet b: null = a\n");
  assert_correct!("let a: null | number\na = 5\na = null");
  assert_correct!("let a: null | number\na = 5 && null\n");
  assert_fails!("let a: number = true");
}

#[test]
fn unknown_type() {
  assert_fails!("let a: unknownType\n");
}

#[test]
fn variable_not_defined() {
  assert_fails!("a\n");
  assert_fails!("let a\nb\n");
}

#[test]
fn grouping() {
  assert_correct!("let a: number = (42)");
  assert_correct!("let a: boolean = (true)");
  assert_correct!("let a: string = ('string')");
}

#[test]
fn unary() {
  assert_correct!("let a: number = -42");
  assert_correct!("let a: boolean = !true");
  assert_correct!("let a: boolean = !false");
  assert_correct!("let a: boolean = !'string'");
  assert_correct!("let a: boolean = !7");

  assert_fails!("-true");
  assert_fails!("-'hello'");
  assert_fails!("-null");
}

#[test]
fn assignment() {
  assert_correct!("let a = 42\nlet b: number = a = 5\n");
  assert_correct!("let a = true\nlet b: boolean = a = false\n");
  assert_correct!("let a = null\nlet b: null = a = null\n");
  assert_correct!("let a\nlet b: null = a = null\n");

  assert_fails!("b = 5\n");
  assert_fails!("let a = 42\na = false\n");
  assert_fails!("let a = false\na = 42\n");
  assert_fails!("let a = 'hello'\na = 15\n");
}

#[test]
fn if_and_while() {
  assert_correct!(
    "
let a = 5
if (a)
  a = 10
else
  a = 20

let b: number  = a
"
  );
  assert_correct!(
    "
let a = 5
while (a)
  a = 0

let b: number = a
"
  );
  assert_fails!(
    "
let a = 5
if (b)
  a = 10

let b: number = a
"
  );
}

#[test]
fn binary() {
  assert_correct!("let a: number = 5 - 5");
  assert_correct!("let a: number = 5 / 5");
  assert_correct!("let a: number = 5 * 5");
  assert_correct!("let a: boolean = 5 == 5");
  assert_correct!("let a: boolean = 5 != 5");
  assert_correct!("let a: boolean = 'hello' == 'world'");
  assert_correct!("let a: boolean = 'hello' != 'world'");
  assert_correct!("let a: boolean = 5 > 5");
  assert_correct!("let a: boolean = 5 < 5");
  assert_correct!("let a: boolean = 'a' >= 'b'");
}

#[test]
fn plus() {
  assert_correct!("let a: number = 5 + 5");
  assert_correct!("let a: string = 'hello' + 'world'");
  assert_correct!("let a = 'hello' && 5\nlet b: number | string = a + a\n");
  assert_fails!("5 + ''");
  assert_fails!("'' + 5");
  assert_fails!("5 + false");
  assert_fails!("null + 5");
  assert_fails!("null + true");
}

#[test]

fn minus() {
  assert_fails!("'a' - 8");
  assert_fails!("8 - 'a'");
  assert_fails!("false - null");
}

#[test]
fn comparison() {
  assert_fails!("5 == 'a'");
  assert_fails!("null != false");
  assert_fails!("5 == false");
}

#[test]
fn nullish_coelesing() {
  assert_correct!("let a: boolean = null ?? false");
  assert_correct!("let a: string = 'hello' ?? null");
  assert_correct!("let a: number = 5 ?? ''");
  assert_correct!("let a: null = null ?? null");
  assert_correct!("let a: number = 5 ?? 6");
  assert_correct!("let a: number  = 5 ?? null");
  assert_correct!("let a: number = null ?? 5");
  assert_correct!("let a: number = 5 ?? false");
}

#[test]
fn and() {
  assert_correct!("let a: number = 5 && 6");
  assert_correct!("let a: number | null = 5 && null");
  assert_correct!("let a: number? = 5 && null");
  assert_correct!("let a: null = null && 5");
  assert_correct!("let a: number | boolean = 5 && false");
  assert_correct!("let a: boolean  = false && 5");
}

#[test]
fn or() {
  assert_correct!("let a: number = 5 || 6");
  assert_correct!("let a: number | null = 5 || null");
  assert_correct!("let a: number = null || 5");
  assert_correct!("let a: number | boolean = 5 || false");
  assert_correct!("let a: number  = false || 5");
}

#[test]
fn call_not_callable() {
  assert_fails!("5()");
  assert_fails!("'hello'()");
  assert_fails!("true()");
  assert_fails!("null()");
}

#[test]
fn call() {
  assert_correct!("let a: null = (() => null)()");
  assert_correct!("let b: string = (() => 'hello')()");
  assert_correct!("let c: number = ((a: number, b: number) => a + b)(7, 8)");
  assert_correct!(
    "
let not = (x: any) => !x
let a: boolean = not(true)
let b: boolean = not(false)
let c: boolean = not(null)
let d: boolean = not(3.5)
      "
  );
  assert_correct!(
    "
let not = (x) => !x
let a: boolean = not(true)
let b: boolean = not(false)
let c: boolean = not(null)
let d: boolean = not(3.5)
      "
  );
  assert_correct!(
    "
let identity = (x) => x
let a: boolean = identity(true)
let b: boolean = identity(false)
let c: null = identity(null)
let d: number = identity(3.5)
      "
  );
  assert_fails!("((a: number, b: number) => a + b)(7)");
  assert_fails!("((a: number, b: number) => a + b)(7, 8, 9)");
  assert_fails!("((a: number, b: number) => a + b)(7, false)");
  assert_fails!("((a: number, b: number) => a + b)(7, null)");
}

#[test]
fn functions() {
  assert_correct!("let func: (number, number) -> number = (a: number, b: number) => a + b");
  assert_correct!("let a: (string) -> string = print");
  assert_correct!("let a: ((any) -> null) | ((any) -> string) = type");
  assert_correct!("let p: (any) -> any = print\nlet t: (any) -> string = type\n");
  assert_correct!(
    "let func: (number | string) -> number | string = (a: number | string | boolean) => 7"
  );
  assert_fails!("let func: (number, string) -> number  = (a: number, b: string) => a || b");
}

#[test]
fn recursive() {
  assert_correct!(
    "
let fib_recursive = (n: number) -> number
  if (n <= 2)
    if (n == 0) return 0
    return n - 1
  else return fib_recursive(n - 1) + fib_recursive(n - 2)

let a: number = fib_recursive(25)
"
  );
}

#[test]
#[should_panic]
fn corecursive() {
  assert_correct!(
    "
let a = (n: number) -> number
  if (n > 0)
    return b(n)
  return n

let b = (n: number) -> number
  return a(n-1)

let c: number = b(5)
"
  );
}

#[test]
fn imports() {
  assert_correct!(
    r"
from string import { trim }
let a: (string) -> string = trim"
  );
  assert_correct!(
    r"
from string import { toNumber }
let a: (string) -> number? = toNumber"
  );
  assert_correct!(
    r"
from string import { includes }
let a: (string, string) -> boolean = includes
"
  );

  assert_correct!(
    r"
from maths import { pow }
let a: (number, number) -> number = pow
"
  );
  assert_correct!(
    r"
from maths import { sin }
let a: (number) -> number = sin
"
  );
}

#[test]
fn returns() {
  assert_correct!(
    "
let numbers = (n: number) -> number
  if (n <= 2)
    if (n == 0) return 0
    return n - 1
  else return n * 5

let a: number = numbers(25)
"
  );
  assert_correct!(
    "
let numbers = (n: number) ->
  if (n <= 2)
    if (n == 0) return 0
    return n - 1
  else return n * 5

let a: number = numbers(25)
"
  );
  assert_correct!(
    "
let x = (n: number) ->
  let a = 7

let a: null = x(6)
"
  );
  assert_fails!(
    "
let numbers = (n: number) -> number
  if (n <= 2)
    if (n == 0) return 0
    return n - 1
  else return n * 5 || 'hello'

numbers(25)
  "
  );
}

#[test]
fn pipeline() {
  assert_correct!(
    "
let add_one = (a: number) => a + 1

let a:number = 3 >> add_one
"
  );
  assert_correct!(
    "
let add_one = (a: number) => a + 1

let a:number = 3 >> add_one()
"
  );
  assert_correct!(
    "
let add = (a: number, b: number) => a + b
let multiply = (a: number, b: number) => a * b

let a:number = 3 >> add(4) >> multiply(5)
    "
  );
}

#[test]
fn redefined_variables() {
  assert_correct!(
    "
let a = false
  let a = 5
  a = -a
"
  );
}

mod narrowing {
  use super::*;

  #[test]
  fn not_equals() {
    assert_correct!(
      "
let func = (a: number?) ->
  if (a != null) -a
"
    );
    assert_correct!(
      "
let s = (b: string) => b
let func = (a: string) ->
  if (a != '') s(a)
  "
    );
  }

  #[test]
  fn equals() {
    assert_correct!(
      "
let boolean = (b: boolean) => b
let func = (a: boolean?) ->
  if (a == true) boolean(a)
"
    );
    assert_correct!(
      "
let s = (b: string) => b
let func = (a: string?) ->
  if (a == '') s(a)
  "
    );
  }

  #[test]
  fn or() {
    assert_correct!(
      "
let boolean = (b: boolean) => b
let func = (a: boolean?) ->
  if (a == true || a == false) boolean(a)

"
    );
  }

  #[test]
  fn and() {
    assert_correct!(
      "
let boolean = (b: boolean) => b
let func = (a: boolean?) ->
  if (a == true && a == false) boolean(a)

"
    );
  }

  #[test]
  fn multiple_and() {
    assert_correct!(
      "
let boolean = (b: boolean) => b
let func = (a: boolean?, b: boolean?) ->
  if (a == true && b == false)
    boolean(a)
    boolean(b)
"
    );
  }

  #[test]
  fn multiple_or() {
    assert_fails!(
      "
let boolean = (b: boolean) => b

let func = (a: boolean?, b: boolean?) ->
  if (a == true || b == false)
    boolean(a)
    boolean(b)
"
    );
  }

  #[test]
  fn else_() {
    assert_correct!(
      "
let boolean = (b: boolean) => b
let n = (n: null) => null

let func = (a: boolean?) ->
  if (a == true || a == false) boolean(a)
  else n(a)
"
    );
  }
}

#[test]
fn list() {
  assert_correct!("[1, 2, 3]");
  assert_correct!("[null, false, true]");
  assert_correct!("let a: (number | string)[] = [1, 2, 3]");
  assert_correct!("let a: (number | string)[] = [1, 'hello', 3]");
  assert_fails!("let a: (number | string)[] = [1, null, 3]");
}

#[test]
fn list_builtins() {
  assert_correct!(
    "
from list import { length, isEmpty, get, pop, length, push, includes, reverse }

let a: number? = [1, 2, 3] >> get(0)
let b: number? = [1, 2, 3] >> pop()
let c: (number[]) -> number = length
let d: boolean = isEmpty([])
let e: number[] = [1, 2, 3] >> push(7)
let f: (number[], number) -> boolean = includes
let g: (number | string)[] = [1, 'hello', 3] >> reverse()
"
  );
}
