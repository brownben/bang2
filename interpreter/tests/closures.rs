mod bang_test;
use bang_test::*;

bang_test!(captures_local
"
let outer = () ->
  let a = 77
  let b = () => a
  return b()

let x = outer()
"
  x == 77
);

bang_test!(captures_local_and_returns_closure
"
let outer = () ->
  let a = 77
  let b = () => a
  return b

let x = outer()()
"
  x == 77
);

bang_test!(captures_local_and_sets
"
let outer = () ->
  let a = 77
  let b = () => a = 66
  b()
  return a

let x = outer()
"
  x == 66
);

bang_test!(capture_variable_multiple_times
"
let outer = () ->
  let a = 1
  let b = () => a + 1
  let c = () => a + 2
  return b() + c()

let x = outer()
"
  x == 5
);

bang_test!(capture_variable_set_multiple_times
"
let outer = () ->
  let a = 0
  let b = () => a += 1
  let c = () => a += 2
  b() + c()
  return a

let x = outer()
"
  x == 3
);

bang_test!(getter_setter
"
let factory = () ->
  let value = 0
  let get = () => value
  let set = (x) => value = x
  return [get, set]

let [get, set] = factory()
let a = get()
set(4)
let b = get()
"
  a == 0
  b == 4
);

bang_test!(equality
  "
let factory = () ->
  let value = 0
  let get = () => value
  let set = (x) => value = x
  return [get, set]

let [get, set] = factory()
let [get2, set2] = factory()

let a = get == get
let b = get != get2
let c = get != set
"
  a == true
  b == true
  c == true
);

bang_test!(modify_captured_variable
"
let outer = () ->
  let a = 1
  let get = () => a
  a = 8
  return get

let x = outer()()
"
  x == 8
);

bang_test!(string_representation
"
let outer = () ->
  let a = 1
  let get = () => a
  a = 8
  return get

let x = '${outer()}'
"
  x == "<function get>"
);

bang_test!(function_closed_with_parameter
"
let run = (do) => do()

let x = (y) -> null
  let n = 7
  return run(() => n)

let a = x(44)
"
  a == 7
);

bang_test!(higher_scope_manual
"
let a = () ->
  let alpha = 77

  let b = () ->
    let alpha = alpha
    return () => alpha

  return b

let x = a()()()
"
  x == 77
);

bang_test!(higher_scope
"
let a = () ->
  let alpha = 77

  let b = () ->
    let c = () => alpha
    return c

  return b

let x = a()()()
"
  x == 77
);

bang_test!(set_higher_scope
"
let a = () ->
  let alpha = 77

  let b = () => () => alpha = 5
  b()()

  return alpha

let x = a()
"
  x == 5
);

bang_test!(double_reference
"
let a = () ->
  let alpha = 8

  let b = () ->
    let c = () => alpha + alpha
    return c

  return b

let x = a()()()
"
x == 16
);
