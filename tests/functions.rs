mod bang_test;
use bang_test::*;

bang_test!(function_without_return_returns_null
"
fun test()
  4 + 5

let resultIsNull = test() == null
"
  resultIsNull == true
);

bang_test!(return_no_value
"
fun test()
  return

let resultIsNull = test() == null
"
  resultIsNull == true
);

bang_test!(function_returns_value
"
fun test()
  return 4 + 5

let result = test()
"
  result == 9.0
);

bang_test!(function_doesnt_execute_after_return
"
let i = 0
fun test()
  return 4 + 5
  i = 7 + 5 + 3

let result = test()
"
  i == 0.0
  result == 9.0
);

bang_test!(function_accepts_parameters
"
fun test(a, b)
  return a + b

let result = test(3, 5)

fun test(a, b,)
  return a + b

let result2 = test(100, 5)
"
  result == 8.0
  result2 == 105.0
);

bang_test!(function_errors_when_too_many_arguments
"
fun test(a, b)
  return a + b

test(3, 5, 8)
"
  RuntimeError
);

bang_test!(function_errors_when_too_few_arguments
"
fun test(a, b)
  return a + b

test(3)
"
  RuntimeError
);

bang_test!(iterative_fibonacci
"
fun fib_iterative(n)
  let x = 0
  let y = 1
  let i = 1
  while (i < n)
    let z = x + y
    x = y
    y = z
    i += 1
  return x

let result = fib_iterative(36)
"
  result == 9227465.0
);

bang_test!(recursive_fibonacci
"
fun fib_recursive(n)
  if (n == 0)
    return 0
  else if (n <= 2)
    return n - 1
  else
    return fib_recursive(n - 1) + fib_recursive(n - 2)

let result = fib_recursive(25)
"
  result == 46368.0
);

bang_test!(recursive_loop
"
let iterations = 0

fun loop(n)
  iterations += 1
  if (n == 0)
    return 0
  else
    return loop(n - 1)

let result = loop(10)
"
  result == 0.0
  iterations == 11.0
);

bang_test!(function_as_argument
  "
fun identity(x)
  return x

fun f(a)
  return a(4) * 4

let result = f(identity)
"
  result == 16.0
);

bang_test!(cant_call_number
  "
1()
"
  RuntimeError
);

bang_test!(cant_call_string
  "
'hello'()
"
  RuntimeError
);

bang_test!(cant_call_null
  "
null()
"
  RuntimeError
);

bang_test!(cant_call_boolean
  "
true()
"
  RuntimeError
);

