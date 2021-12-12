mod bang_test;
use bang_test::*;

bang_test!(add_numbers
"
let a = 1 + 2
let b = 3.3 + 4.2
let c = 1000.2 - 35.7
"
  a == 3.0
  b == 7.5
  c == 964.5
);

bang_test!(subtract_numbers
"
let a = 5 - 2
let b = 10.1 - 2.6
let c = 964.99 - .49
"
  a == 3.0
  b == 7.5
  c == 964.5
);

bang_test!(multiply_numbers
"
let a = 5 * 2
let b = 5.0 * 2.0
let c = 3.2 * 2
let d = 0.5 * 16.4
"
  a == 10.0
  b == 10.0
  c == 6.4
  d == 8.2
);

bang_test!(divide_numbers
"
let a = 5 / 2
let b = 5.0 / 2.0
let c = 22 / 11
"
  a == 2.5
  b == 2.5
  c == 2.0
);

bang_test!(negate_numbers
"
let a = - 2
let b = - 2.6
let c = -4525
"
  a == -2.0
  b == -2.6
  c == -4525.0
);

bang_test!(assignment_operators
"
let a = 10
let b = 10
let c = 10
let d = 10

a += 2
b -= 2
c /= 2
d *= 2
"
  a == 12.0
  b == 8.0
  c == 5.0
  d == 20.0
);

bang_test!(concatenate_strings
"
let a = \"Hello \" + \"World\"
let b = 'Whats Up' + `?`
let c = \"Merged\" + 'together'
"
  a == "Hello World"
  b == "Whats Up?"
  c == "Mergedtogether"
);

bang_test!(lots_of_minuses
"
let a = 4---4
let b = 4--------------------------------4
"
  a == 0.0
  b == 8.0
);

bang_test!(cant_add_string_and_number
"
\"Hello\" + 4
"
  RuntimeError
);

bang_test!(cant_add_boolean_and_number
"
true + 4
"
  RuntimeError
);

bang_test!(cant_minus_boolean_and_number
"
4 - false
"
  RuntimeError
);

bang_test!(cant_multiply_string
"
\"Hello\" * 4
"
  RuntimeError
);

bang_test!(cant_divide_null
"
null / 45
"
  RuntimeError
);

bang_test!(bidmas
"
let a = (5 - (3 - 1)) + -1
let b = 5 + 3 * 2
"
  a == 2.0
  b == 11.0
);

bang_test!(cant_add_boolean_null
"
false + null
"
  RuntimeError
);
