mod bang_test;
use bang_test::*;

bang_test!(type_of
"
let a = type(3)
let b = type(true)
let c = type(null)
let d = type(type)
let e = type('hello')
"
  a == "number"
  b == "boolean"
  c == "null"
  d == "function"
  e == "string"
);

bang_test!(type_correct_number_of_arguments
"
type(1,2)
"
  RuntimeError
);

bang_test!(print
"
let a = print(3)
let aIsNull = a == null
"
  aIsNull == true
);
