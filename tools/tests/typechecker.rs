use bang_syntax::parse;
use bang_tools::typecheck;

macro_rules! assert_correct {
  ($source:expr) => {
    let ast = parse($source).unwrap();
    let result = typecheck(&ast);

    assert!(result.is_empty(), "{result:?}");
  };
}

macro_rules! assert_fails {
  ($source:expr) => {
    let ast = parse($source).unwrap();
    let result = typecheck(&ast);

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
  assert_correct!("let a: string = 'Hello, ${2 + 5}!'");
}

#[test]
fn unknown_type() {
  assert_fails!("let a: unknownType\n");
}

#[test]
fn grouping() {
  assert_correct!("let a: number = (42)");
  assert_correct!("let a: boolean = (true)");
  assert_correct!("let a: string = ('string')");
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

  assert_correct!(
    "
let a = (x: number) ->
  if (x > 5)
    return 4
let b: number? = a(44)
"
  );

  assert_correct!(
    "
let a = (x: number) ->
  if (x > 5)
    return 4
  return

let b: number? = a(44)
"
  );

  assert_fails!("while (true) 4");
}

#[test]
fn dead_code_ignored() {
  assert_correct!(
    "
let a: () -> null = () ->
  if (false) return 7

let b: () -> number = () ->
  if (true) return 7
  "
  );
  assert_correct!(
    "
let a: () -> string = () ->
  if (false) return 7
  else return ''

let b: () -> number = () ->
  if (true) return 7
  else return ''
  "
  );
  assert_correct!(
    "
let a: () -> null = () ->
  while (false) return 7

let b: () -> number = () ->
  while (true) return 7

let c: () -> number | null = () ->
  while (a) return 7

let d: () -> number | string = () ->
  let a: number = 0
  while (a) return 7
  return ''
  "
  );

  assert_correct!(
    "
let a: () -> null = () ->
  return null
  return 7
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

  assert_fails!("from maths import { unknown }");
  assert_fails!("from list import { unknown }");
  assert_fails!("from unknown import { unknown }");

  assert_correct!("let a: (number, number) -> number = maths::pow");
  assert_fails!("let a = maths::x");
  assert_fails!("let a = unknown::x");
}

#[test]
fn unions() {
  assert_correct!(
    "
  let a: any | number
  let b: any = a
    "
  );
  assert_correct!(
    "
let a = (x: (number[] | string[]) | (number | string)) => x
a([5])
a(5)
a('hello')
  "
  );
}

mod variables {
  use super::*;

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
    assert_fails!("let a: number // as initialized to null");
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
    assert_fails!(
      "
  let a = false
  let a = 5
  a = -a
  "
    );
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

    assert_correct!("let a: number | null = 4\n let b: number = a");
    assert_correct!("let a: number | null = 4\n a = null\n let b: null = a");
  }

  #[test]
  fn variable_not_defined() {
    assert_fails!("a\n");
    assert_fails!("let a\nb\n");
  }

  #[test]
  fn list_destructuring() {
    assert_correct!(
      "
let [x, y, z] = [0, 1, 2]
let a: number = x
let b: number = y
let c: number = z
    "
    );
    assert_correct!("let [a, b]: number[] = [1, 2, 3, 4]");
    assert_fails!("let [a, b] = null");
    assert_correct!(
      "
let [a, b]: string = 'hello'
let c: string = a
"
    );
    assert_fails!("let [a, b] = 373.32");
  }
}

mod operators {
  use super::*;

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
    assert_correct!(
      "
  let add = (a: number, b: number) => a + b
  let multiply = (a: number, b: number) => a * b

  let a:number = 3 >> add(4) >> multiply(5) // comment to unwrap
      "
    );
  }

  #[test]
  fn truthyness_with_unions() {
    assert_correct!(
      "
  let func: ((any) -> any) | ((number) -> number) = print
  let a: false = !func"
    );
    assert_correct!(
      "
  let func: ((any) -> any) | ((number) -> number) = print
  let a: ((any) -> any) | ((number) -> number) = func || 7"
    );
  }
}

mod functions {
  use super::*;

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
  #[ignore = "use of globals before definition is not supported yet"]
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
    assert_correct!(
      "
let x = (n) -> number | null | boolean | string
  if (n == 'hello') return 5
  if (n == 'hello') return null
  if (n == 'hello') return false
  else return ''

let a: number | null | boolean | string = x('hi')
"
    );
    assert_fails!(
      "
let x = (n) ->
  if (n == 'hello') return 5
  if (n == 'hello') return null
  if (n == 'hello') return false
  else return ''

let a: number = x('hi')
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
  fn call_existential() {
    assert_correct!(
      "
let func = (a) ->
  return a

let a: (number) -> number = func
"
    );
    assert_correct!(
      "
let plusFiveAdder = (number, adder) ->
  return adder(number + 5)

let adder = (number) => number + 1

let d: number = plusFiveAdder(5, adder)
"
    );
  }
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
    assert_correct!(
      "
let s = (b: true) => b
let func = (a: boolean) ->
  if (a != false) s(a)
  "
    );
    assert_correct!(
      "
let s = (b: true) => b
let func = (a: boolean) ->
  if ((a != false)) s(a)
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

  #[test]
  fn to_never() {
    assert_fails!(
      "
let boolean = (b: boolean) ->
  if (b == true)
    return
  else if (b == false)
    return
  else
    boolean(b)
"
    );
  }

  #[test]
  fn to_union() {
    assert_correct!(
      "
let boolean = (b: boolean?) ->
  if (b == true)
    return
  else if (b == false)
    return
  else
    b == null
"
    );
  }
}

mod compound_structures {
  use super::*;

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
  from list import { length, isEmpty, get, pop, push, includes, reverse }

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

  #[test]
  fn set_builtins() {
    assert_correct!(
      "
  from set import { set, size, isEmpty, insert, remove, includes, isDisjoint, isSuperset, isSubset, union, difference, intersection, symmetricDifference }

  let a = set(1, 2, 3)
  let b: number = a >> size()
  let c: boolean = a >> isEmpty()
  let d: boolean = a >> insert(2)
  let e: boolean = a >> remove(3)
  let f: boolean = a >> includes(4)
  let g = union(a, set(1,2,3))
  let h = difference(a, g)
  let i = intersection(a, g)
  let j = symmetricDifference(h, i)
  "
    );
  }

  #[test]
  fn index() {
    assert_correct!("let a: string = 'hello'[5]");
    assert_fails!("let a: string = 'hello'[null] ");
    assert_fails!("let a: string = null[5] ");
    assert_fails!("let a: string = boolean[null] ");

    assert_correct!("let a: number | string = [1, 2, 3][1]");
    assert_correct!("let a: number = [1, 2, 3][1]");
    assert_correct!("let a: number | string = [1, 2, 3, 'hello'][1]");
    assert_fails!("let a: number | string = [1, 2, 3]['hello']");
    assert_fails!("let a: number | string = [1, 2, null, 3]['hello']");
    assert_correct!("let a: number[] | string[] = [1, 2, 3]\n let b: number | string = a[0]");
  }

  #[test]
  fn index_assignment() {
    assert_correct!("let a = [1, 2, 3]\n a[0] = 4");
    assert_fails!("let a = [1, 2, 3]\n a[''] = 4");
    assert_fails!("let a = [1, 2, 3]\n a[0] = null");

    assert_correct!("let a: number | string = [1, 2, 3][1]");
    assert_correct!("let a: number = [1, 2, 3][1]");
    assert_correct!("let a: number | string = [1, 2, 3, 'hello'][1]");
    assert_fails!("let a: number | string = [1, 2, 3]['hello']");
    assert_fails!("let a: number | string = [1, 2, null, 3]['hello']");
  }

  #[test]
  fn index_assignment_operator() {
    assert_correct!("let a = [1, 2, 3]\n a[0] += 4");
    assert_fails!("let a = [1, 2, 3]\n a[''] += 4");
    assert_fails!("let a = [1, 2, 3]\n a[0] += null");
    assert_correct!("let a = [1, 5, 3]\n a[2] -= 8");
    assert_fails!("let a: (boolean|number)[] = [1, false, 3]\n a[0] += 4");
  }

  #[test]
  fn dict() {
    assert_correct!("let a: dict(string, number) = dict::new()");
    assert_correct!("let a: dict(string, number) = dict::new()\nlet b: number = a['hello']");
    assert_correct!(
      "
let a = (a) ->
  let b = a[null]
  dict::size(a)
"
    );

    assert_fails!("let a: dict(string, number) = dict::new()\nlet b: number = a[boolean]");
    assert_fails!("let a: dict(string, number) = dict::new()\nlet b: number = a[7]");
    assert_fails!("let a: dict(string, number) = []");
    assert_fails!("let a: list(string) = dict::new()");
    assert_fails!("let a: unknown(string, string) = dict::new()");
    assert_fails!("let a: dict(string) = dict::new()");
  }
}

mod varadic_arguments {
  use super::*;

  #[test]
  fn catch_all_arguments() {
    assert_correct!("let a = (..x) => 7\na(1,2,3)");
    assert_correct!("let a = (..x) => 7\na()");
    assert_correct!("let a = (..x) => 7\na(1)");
  }

  #[test]
  fn catch_all_arguments_assert_type() {
    assert_correct!("let a = (..x: number) => 7\na(1,2,3)");
    assert_correct!("let a = (..x: number) => 7\na()");
    assert_correct!("let a = (..x: number) => 7\na(1)");
    assert_fails!("let a = (..x: number) => 7\na('')");
  }

  #[test]
  fn call_catch_remaining() {
    assert_correct!(
      "
from list import { length }
let a = (x, ..catch) => length(catch)
let b = a(1, 2, 3)
let c = a(2)
let d = a(2,3,3,5,8)
"
    );

    assert_fails!(
      "
from list import { length }
let a = (x, ..catch) => length(catch)
let b = a()
"
    );
  }

  #[test]
  fn catch_remaining_is_list() {
    assert_fails!(
      "
from list import { length }
let a = (x: number, ..catch) => catch + 7
let b = a(1, 2, 3)
"
    );
    assert_correct!(
      "
from list import { length }
let a = (x: number, ..catch: number) => 7
let b = a(1, 2, 3)
"
    );
    assert_correct!(
      "
let a = (x, ..catch) => catch
let b: string[] = a(7, '2', 'hello')
"
    );
    assert_correct!(
      "
let a = (..catch) => catch[0]
let b: string | number = a(7, '2', 'hello')
"
    );
  }
}
