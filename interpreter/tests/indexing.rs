mod bang_test;
use bang_test::*;

bang_test!(list_index
  "
let a = [1, 2, 3][0]
let b = [1, 2, 3][1]
let c = [1, 2, 3][2]
"
  a == 1
  b == 2
  c == 3
);

bang_test!(string_index
"
let a = 'hello'[0]
let b = 'hello'[1]
let c = 'hello'[2]
let d = 'hello'[4]
"
  a == "h"
  b == "e"
  c == "l"
  d == "o"
);

bang_test!(negative_index
  "
let a = 'hello'[-1]
let b = 'hello'[-0]
let c = 'hello'[-5]
"
  a == "o"
  b == "h"
  c == "h"
);

bang_test!(index_rounding
  "
let a = 'hello'[0.01]
let b = 'hello'[0.99]
let c = 'hello'[1.5]
let d = 'helLo'[2.5]
let e = 'helLo'[-1.5]
let f = 'hello'[3.68]
"
  a == "h"
  b == "e"
  c == "l"
  d == "L"
  e == "L"
  f == "o"
);

bang_test!(string_too_large_index
  "'hello'[77]"
  RuntimeError
);

bang_test!(string_too_negative_index
  "'hello'[-77]"
  RuntimeError
);

bang_test!(list_too_large_index
  "[4, 5, 6][77]"
  RuntimeError
);

bang_test!(list_too_negative_index
  "[1, 2, 3][-77]"
  RuntimeError
);

bang_test!(cant_index_number
  "88[-77]"
  RuntimeError
);

bang_test!(cant_index_bool
  "false[-77]"
  RuntimeError
);

bang_test!(cant_index_function
  "print[0]"
  RuntimeError
);

bang_test!(cant_index_null
  "null[-77]"
  RuntimeError
);

bang_test!(cant_index_list_with_null
  "[1, 2, 3][null]"
  RuntimeError
);

bang_test!(cant_index_assign_list_with_null
  "[1, 2, 3][null] = 9"
  RuntimeError
);

bang_test!(cant_index_string_with_null
  "'hello'[null]"
  RuntimeError
);

bang_test!(cant_assign_to_number_index
  "1[2] = 3"
  RuntimeError
);

bang_test!(cant_assign_to_string_index
  "'hello'[0] = 'H'"
  RuntimeError
);

bang_test!(cant_assign_to_element_not_in_list
  "[1, 2, 3][5] = 6"
  RuntimeError
);

bang_test!(assign_to_list
  "
let list = [1, 2, 3]

let a = list[0]
let b = list[1]
let c = list[2]

list[0] = 4
list[1] = 5
list[2] = 6

let d = list[0]
let e = list[1]
let f = list[2]
  "
  a == 1
  b == 2
  c == 3
  d == 4
  e == 5
  f == 6
);

bang_test!(list_index_assignment_operator
  "
let list = [1, 2, 3, 4]

list[0] += 5
list[1] -= 5
list[2] *= 5
list[3] /= 5

let a = list[0]
let b = list[1]
let c = list[2]
let d = list[3]
  "
  a == 6
  b == -3
  c == 15
  d == 0.8
);

bang_test!(index_assignment_operator_index_evaluated_once
  "
let list = [1, 2, 3, 4]
let a = 0

list[a += 1] += 5

let a = list[0]
let b = list[1]
let c = list[2]
let d = list[3]
  "
  a == 1
  b == 7
  c == 3
  d == 4
);

bang_test!(list_destructuring
  "
let [a, b, c] = [5, 6, 7]
let [d] = [8]
let [e, f] = [9, 10]

let list = [4, 1, 8]
let [x, y, z] = list
  "
  a == 5
  b == 6
  c == 7
  d == 8
  e == 9
  f == 10

  x == 4
  y == 1
  z == 8
);

bang_test!(list_destructuring_too_few
  "let [a, b, c] = [5, 6]\n"
  RuntimeError
);

bang_test!(list_destructuring_too_many
  "
let [a, b, c] = [5, 6, 7, 8, 9]

let list = [4, 1, 8, 6, 5]
let [x, y, z] = list
  "
  a == 5
  b == 6
  c == 7

  x == 4
  y == 1
  z == 8
);
