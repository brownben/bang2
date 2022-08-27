mod bang_test;
use bang_test::*;

bang_test!(top_level_return
"
let a = 8
return
a = 4
"
  a == 8
);

bang_test!(top_level_return_with_value
"
let a = 8
return 'hello'
a = 4
"
  a == 8
);

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
let test = () -> number
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
let test = () -> number
  return 4 + 5
  i = 7 + 5 + 3

let result = test()
"
  i == 0.0
  result == 9.0
);

bang_test!(function_accepts_parameters
"
let test = (a: number, b: number) -> number
  return a + b

let result = test(3, 5)

let test = (a: number, b: number,) -> number
  return a + b

let result_two = test(100, 5)
"
  result == 8.0
  result_two == 105.0
);

bang_test!(function_errors_when_too_many_arguments
"
let test = (a: number, b: number,) -> number
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
let test = (a: number, b: number,) -> number
  return a + b

test(3)
"
  RuntimeError
);

bang_test!(iterative_fibonacci
"
let fib_iterative = (n: number) -> number
  let x = 0
  let y = 1
  let i = 1
  while (i < n)
    let z = x + y
    x = y
    y = z
    i += 1
  return x

let result = fib_iterative(10)
"
  result == 34
);

bang_test!(recursive_fibonacci
"
let fib_recursive = (n: number) -> number
  if (n == 0)
    return 0
  else if (n <= 2)
    return n - 1
  else
    return fib_recursive(n - 1) + fib_recursive(n - 2)

let result = fib_recursive(5)
"
  result == 3
);

bang_test!(recursive_loop
"
let iterations = 0

let loop = (n: number) -> number
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
let identity = (x: number) => x

let f = (a: (number) -> number) -> number
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
let a = (
  x0: number, x1: number, x2: number, x3: number, x4: number, x5: number, x6: number, x7: number, x8: number, x9: number, xa: number, xb: number, xc: number, xd: number, xe: number, xf: number,
  x10: number, x11: number, x12: number, x13: number, x14: number, x15: number, x16: number, x17: number, x18: number, x19: number, x1a: number, x1b: number, x1c: number, x1d: number, x1e: number, x1f: number,
  x20: number, x21: number, x22: number, x23: number, x24: number, x25: number, x26: number, x27: number, x28: number, x29: number, x2a: number, x2b: number, x2c: number, x2d: number, x2e: number, x2f: number,
  x30: number, x31: number, x32: number, x33: number, x34: number, x35: number, x36: number, x37: number, x38: number, x39: number, x3a: number, x3b: number, x3c: number, x3d: number, x3e: number, x3f: number,
  x40: number, x41: number, x42: number, x43: number, x44: number, x45: number, x46: number, x47: number, x48: number, x49: number, x4a: number, x4b: number, x4c: number, x4d: number, x4e: number, x4f: number,
   x50: number, x51: number, x52: number, x53: number, x54: number, x55: number, x56: number, x57: number, x58: number, x59: number, x5a: number, x5b: number, x5c: number, x5d: number, x5e: number, x5f: number,
   x60: number, x61: number, x62: number, x63: number, x64: number, x65: number, x66: number, x67: number, x68: number, x69: number, x6a: number, x6b: number, x6c: number, x6d: number, x6e: number, x6f: number,
   x70: number, x71: number, x72: number, x73: number, x74: number, x75: number, x76: number, x77: number, x78: number, x79: number, x7a: number, x7b: number, x7c: number, x7d: number, x7e: number, x7f: number,
   x80: number, x81: number, x82: number, x83: number, x84: number, x85: number, x86: number, x87: number, x88: number, x89: number, x8a: number, x8b: number, x8c: number, x8d: number, x8e: number, x8f: number,
   x90: number, x91: number, x92: number, x93: number, x94: number, x95: number, x96: number, x97: number, x98: number, x99: number, x9a: number, x9b: number, x9c: number, x9d: number, x9e: number, x9f: number,
   xa0: number, xa1: number, xa2: number, xa3: number, xa4: number, xa5: number, xa6: number, xa7: number, xa8: number, xa9: number, xaa: number, xab: number, xac: number, xad: number, xae: number, xaf: number,
   xb0: number, xb1: number, xb2: number, xb3: number, xb4: number, xb5: number, xb6: number, xb7: number, xb8: number, xb9: number, xba: number, xbb: number, xbc: number, xbd: number, xbe: number, xbf: number,
   xc0: number, xc1: number, xc2: number, xc3: number, xc4: number, xc5: number, xc6: number, xc7: number, xc8: number, xc9: number, xca: number, xcb: number, xcc: number, xcd: number, xce: number, xcf: number,
   xd0: number, xd1: number, xd2: number, xd3: number, xd4: number, xd5: number, xd6: number, xd7: number, xd8: number, xd9: number, xda: number, xdb: number, xdc: number, xdd: number, xde: number, xdf: number,
   xe0: number, xe1: number, xe2: number, xe3: number, xe4: number, xe5: number, xe6: number, xe7: number, xe8: number, xe9: number, xea: number, xeb: number, xec: number, xed: number, xee: number, xef: number,
   xf0: number, xf1: number, xf2: number, xf3: number, xf4: number, xf5: number, xf6: number, xf7: number, xf8: number, xf9: number, xfa: number, xfb: number, xfc: number, xfd: number, xfe: number, xff: number, xaa) -> number
  return 1
"
  CompileError
);

bang_test!(call_trailing_comma
"
let a = print(1,)
let b = print('hello')
"
  a == 1
  b == "hello"
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
let factorial = (n: number) -> number
  if (n == 0) return 1
  else return factorial(n-1) * n

let a = factorial(15)
"
  a == 1307674368000.0
);

bang_test!(factorial_tailcall
"
let factorial = (n: number, a: number) -> number
  if (n == 0) return a
  else return factorial(n-1, a * n)

let a = factorial(15, 1)
"
  a == 1307674368000.0
);

bang_test!(tailcall_loop
  "
let tailcall_loop = (n: number) -> number
  if (n == 0) return 0
  else return tailcall_loop(n-1)

let a = tailcall_loop(1000)
"
  a == 0.0
);

bang_test!(parameters_on_different_lines
  "
let test = (
  a: number,
  b: number,
) -> number
  return a + b

let a = test(3, 5)
"
  a == 8.0
);

bang_test!(arguments_on_different_lines
  "
let test = (a: number, b: number) -> number
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
let identity = (x: any) => x

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
let identity = (x: any) => x

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
let add = (x: number, y: number) => x + y
let divide = (x: number, y: number) => x / y

let a = 1 >> add(2)
let b = 4 >> divide(2)
"
  a == 3.0
  b == 2.0
);

bang_test!(pipeline_error_if_too_many_args
  "
let add = (x: number, y: number) => x + y
let divide = (x: number, y: number) => x / y

let a = 1 >> add(1, 2)
"
  RuntimeError
);

bang_test!(pipeline_precendence
  "
let identity = (x: any) => x
let not = (x: any) => !x

let a = 5 or 6 >> identity
let b = false and true >> not >> not
"
  a == 5.0
  b == false
);

bang_test!(pipeline_with_comment
  "
let identity = (x: any) => x

let a = 1 >> type() // comment
let b = 2 >> identity() //comment
"
  a == "number"
  b == 2.0
);

bang_test!(pipeline_over_max_args
  "
1 >> type(
  1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
  1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
  1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
  1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
  1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
  1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
  1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
  1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
  1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
  1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
  1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
)
"
  CompileError
);

bang_test!(call_over_max_args
  "
type(
  1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
  1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
  1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
  1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
  1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
  1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
  1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
  1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
  1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
  1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
  1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
)
"
  CompileError
);

bang_test!(invalid_function_return_type
  "
let a = () -> (+)
  return 6
"
  CompileError
);

bang_test!(catch_all_parameter_must_be_last
  "let a = (..catch, all) => 7"
  CompileError
);

bang_test!(only_one_catch_all
  "let a = (..catch, ..all) => 7"
  CompileError
);

bang_test!(catch_everything
  "
let a = (..catch) => catch[0]
let b = a(1, 2, 3)
let c = a(2)
let d = a(2,3,3)
"
  b == 1
  c == 2
  d == 2
);

bang_test!(catches_correct_number
  "
from list import { length }
let a = (..catch) => length(catch)
let b = a(1, 2, 3)
let c = a(2)
let d = a(2,3,3,5,8)
let e = a()
"
  b == 3
  c == 1
  d == 5
  e == 0
);

bang_test!(catches_remaining
  "
from list import { length }
let a = (x, ..catch) => length(catch)
let b = a(1, 2, 3)
let c = a(2)
let d = a(2,3,3,5,8)
"
  b == 2
  c == 0
  d == 4
);

bang_test!(catches_remaining_too_few
  "
from list import { length }
let a = (x, ..catch) => length(catch)
let b = a()
"
  RuntimeError
);
