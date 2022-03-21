mod bang_test;
use bang_test::*;

bang_test!(function_without_return_returns_null
"
let test = () ->
  4 + 5

let resultIsNull = test() == null
"
  resultIsNull == true
);

bang_test!(return_no_value
"
let test = () ->
  return

let resultIsNull = test() == null
"
  resultIsNull == true
);

bang_test!(function_returns_value
"
let test = () ->
  return 4 + 5

let result = test()

let test_two = () => 4 + 5
let result_two = test_two()
"
  result == 9.0
  result_two == 9.0
);

bang_test!(function_doesnt_execute_after_return
"
let i = 0
let test = () ->
  return 4 + 5
  i = 7 + 5 + 3

let result = test()
"
  i == 0.0
  result == 9.0
);

bang_test!(function_accepts_parameters
"
let test = (a, b) ->
  return a + b

let result = test(3, 5)

let test = (a, b,) ->
  return a + b

let result_two = test(100, 5)
"
  result == 8.0
  result_two == 105.0
);

bang_test!(function_errors_when_too_many_arguments
"
let test = (a, b,) ->
  return a + b

test(3, 5, 8)
"
  RuntimeError
);

bang_test!(native_function_errors_when_too_many_arguments
"
print(3, 5, 8)
"
  RuntimeError
);

bang_test!(function_errors_when_too_few_arguments
"
let test = (a, b,) ->
  return a + b

test(3)
"
  RuntimeError
);

bang_test!(iterative_fibonacci
"
let fib_iterative = (n) ->
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
let fib_recursive = (n) ->
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

let loop = (n) ->
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
let identity = (x) => x

let f = (a) ->
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

bang_test!(max_255_parameters
"
let a = (x0, x1, x2, x3, x4, x5, x6, x7, x8, x9, xa, xb, xc, xd, xe, xf, x10, x11, x12, x13, x14, x15, x16, x17, x18, x19, x1a, x1b, x1c, x1d, x1e, x1f, x20, x21, x22, x23, x24, x25, x26, x27, x28, x29, x2a, x2b, x2c, x2d, x2e, x2f, x30, x31, x32, x33, x34, x35, x36, x37, x38, x39, x3a, x3b, x3c, x3d, x3e, x3f, x40, x41, x42, x43, x44, x45, x46, x47, x48, x49, x4a, x4b, x4c, x4d, x4e, x4f, x50, x51, x52, x53, x54, x55, x56, x57, x58, x59, x5a, x5b, x5c, x5d, x5e, x5f, x60, x61, x62, x63, x64, x65, x66, x67, x68, x69, x6a, x6b, x6c, x6d, x6e, x6f, x70, x71, x72, x73, x74, x75, x76, x77, x78, x79, x7a, x7b, x7c, x7d, x7e, x7f, x80, x81, x82, x83, x84, x85, x86, x87, x88, x89, x8a, x8b, x8c, x8d, x8e, x8f, x90, x91, x92, x93, x94, x95, x96, x97, x98, x99, x9a, x9b, x9c, x9d, x9e, x9f, xa0, xa1, xa2, xa3, xa4, xa5, xa6, xa7, xa8, xa9, xaa, xab, xac, xad, xae, xaf, xb0, xb1, xb2, xb3, xb4, xb5, xb6, xb7, xb8, xb9, xba, xbb, xbc, xbd, xbe, xbf, xc0, xc1, xc2, xc3, xc4, xc5, xc6, xc7, xc8, xc9, xca, xcb, xcc, xcd, xce, xcf, xd0, xd1, xd2, xd3, xd4, xd5, xd6, xd7, xd8, xd9, xda, xdb, xdc, xdd, xde, xdf, xe0, xe1, xe2, xe3, xe4, xe5, xe6, xe7, xe8, xe9, xea, xeb, xec, xed, xee, xef, xf0, xf1, xf2, xf3, xf4, xf5, xf6, xf7, xf8, xf9, xfa, xfb, xfc, xfd, xfe, xff, xaa) ->
  return 1
"
  CompileError
);

bang_test!(call_trailing_comma
"
let a = print(1,) == null
"
  a == true
);

bang_test!(blank_return_is_null
"
let b = () ->
  return

let a = b() == null
"
  a == true
);

bang_test!(factorial
"
let factorial = (n) ->
  if (n == 0) return 1
  else return factorial(n-1) * n

let a = factorial(15)
"
  a == 1307674368000.0
);

bang_test!(factorial_tailcall
"
let factorial = (n, a) ->
  if (n == 0) return a
  else return factorial(n-1, a * n)

let a = factorial(15, 1)
"
  a == 1307674368000.0
);

bang_test!(tailcall_loop
  "
let tailcall_loop = (n) ->
  if (n == 0) return 0
  else return tailcall_loop(n-1)

let a = tailcall_loop(1000)
"
  a == 0.0
);

bang_test!(parameters_on_different_lines
  "
let test = (
  a,
  b,
) ->
  return a + b

let a = test(3, 5)
"
  a == 8.0
);

bang_test!(arguments_on_different_lines
  "
let test = (a, b) ->
  return a + b

let a = test(
  3,
  5,
)
"
  a == 8.0
);

bang_test!(pipeline_no_call
  "
let identity = (x) => x

let a = 1 >> type
let b = 2 >> identity
let c = 'hello' >> identity >> identity
"
  a == "number"
  b == 2.0
  c == "hello"
);

bang_test!(pipeline_call_no_args
  "
let identity = (x) => x

let a = 1 >> type()
let b = 2 >> identity()
let c = 'hello' >> identity() >> identity()
"
  a == "number"
  b == 2.0
  c == "hello"
);

bang_test!(pipeline_with_args
  "
let add = (x, y) => x + y
let divide = (x, y) => x / y

let a = 1 >> add(2)
let b = 4 >> divide(2)
"
  a == 3.0
  b == 2.0
);

bang_test!(pipeline_error_if_too_many_args
  "
let add = (x, y) => x + y
let divide = (x, y) => x / y

let a = 1 >> add(1, 2)
"
  RuntimeError
);

bang_test!(pipeline_precendence
  "
let identity = (x) => x
let not = (x) => !x

let a = 5 or 6 >> identity
let b = false and true >> not >> not
"
  a == 5.0
  b == false
);
