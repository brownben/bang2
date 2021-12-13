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

bang_test!(max_255_parameters
"
fun a(x0,x1,x2,x3,x4,x5,x6,x7,x8,x9,xa,xb,xc,xd,xe,xf,x10,x11,x12,x13,x14,x15,x16,x17,x18,x19,x1a,x1b,x1c,x1d,x1e,x1f,x20,x21,x22,x23,x24,x25,x26,x27,x28,x29,x2a,x2b,x2c,x2d,x2e,x2f,x30,x31,x32,x33,x34,x35,x36,x37,x38,x39,x3a,x3b,x3c,x3d,x3e,x3f,x40,x41,x42,x43,x44,x45,x46,x47,x48,x49,x4a,x4b,x4c,x4d,x4e,x4f,x50,x51,x52,x53,x54,x55,x56,x57,x58,x59,x5a,x5b,x5c,x5d,x5e,x5f,x60,x61,x62,x63,x64,x65,x66,x67,x68,x69,x6a,x6b,x6c,x6d,x6e,x6f,x70,x71,x72,x73,x74,x75,x76,x77,x78,x79,x7a,x7b,x7c,x7d,x7e,x7f,x80,x81,x82,x83,x84,x85,x86,x87,x88,x89,x8a,x8b,x8c,x8d,x8e,x8f,x90,x91,x92,x93,x94,x95,x96,x97,x98,x99,x9a,x9b,x9c,x9d,x9e,x9f,xa0,xa1,xa2,xa3,xa4,xa5,xa6,xa7,xa8,xa9,xaa,xab,xac,xad,xae,xaf,xb0,xb1,xb2,xb3,xb4,xb5,xb6,xb7,xb8,xb9,xba,xbb,xbc,xbd,xbe,xbf,xc0,xc1,xc2,xc3,xc4,xc5,xc6,xc7,xc8,xc9,xca,xcb,xcc,xcd,xce,xcf,xd0,xd1,xd2,xd3,xd4,xd5,xd6,xd7,xd8,xd9,xda,xdb,xdc,xdd,xde,xdf,xe0,xe1,xe2,xe3,xe4,xe5,xe6,xe7,xe8,xe9,xea,xeb,xec,xed,xee,xef,xf0,xf1,xf2,xf3,xf4,xf5,xf6,xf7,xf8,xf9,xfa,xfb,xfc,xfd,xfe,xff,xaa)
  return 1
"
  CompileError
);
