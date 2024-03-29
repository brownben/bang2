mod bang_test;
use bang_test::*;

bang_test!(less_than_numbers
"
let a = 3 < 4
let b = 3 < 3
let c = -5 < 5
let d = 10 < -10
"
  a == true
  b == false
  c == true
  d == false
);

bang_test!(greater_than_numbers
"
let a = 3 > 4
let b = 3 > 3
let c = -5 > 5
let d = 10 > -10
"
  a == false
  b == false
  c == false
  d == true
);

bang_test!(less_than_strings
"
let a = 'b' < 'c'
let b = 'b' < 'b'
let c = 'H' < 'h'
let d = 'hello' < 'Hello'
"
  a == true
  b == false
  c == true
  d == false
);

bang_test!(greater_than_strings
"
let a = 'b' > 'c'
let b = 'b' > 'b'
let c = 'H' > 'h'
let d = 'hello' > 'Hello'
"
  a == false
  b == false
  c == false
  d == true
);

bang_test!(less_than_or_equal_to_numbers
"
let a = 3 <= 4
let b = 3 <= 3
let c = -5 <= 5
let d = 10 <= -10
"
  a == true
  b == true
  c == true
  d == false
);

bang_test!(greater_than_or_equal_to_numbers
"
let a = 3 >= 4
let b = 3 >= 3
let c = -5 >= 5
let d = 10 >= -10
"
  a == false
  b == true
  c == false
  d == true
);

bang_test!(less_than_mixed_types
"
let a = 3 < ''
"
  RuntimeError
);

bang_test!(less_than_wrong_type
"
let a = 3 < null
"
  RuntimeError
);

bang_test!(greater_than_mixed_types
"
let a = 3 > ''
"
  RuntimeError
);

bang_test!(greater_than_wrong_type
"
let a = 3 > null
"
  RuntimeError
);

bang_test!(equality_number
"
let a = 3 == 4
let b = 3 == 3
let c = -5 == 5
let d = 0 == -0
let e = 0.3 == (0.1 + 0.2)
let e = [0.3] == [0.1 + 0.2]
"
  a == false
  b == true
  c == false
  d == true
  e == true
);

bang_test!(equality_strings
"
let a = 'hello' == \"hello\"
let b = 'hello' == \"goodbye\"
let c = 'hello' == 'hello!'
let d = \"hello\" == \"hello\"
let e = \"hello\" == \"hel\" + \"lo\"
"
  a == true
  b == false
  c == false
  d == true
  e == true
);

bang_test!(equality_boolean
"
let a = true == true
let b = true == false
let c = false == false
"
  a == true
  b == false
  c == true
);

bang_test!(equality_null
"
let a = null == null
"
  a == true
);

bang_test!(no_equality_between_types
"
let a = null == false
let b = 0 == false
let c = null == 0
let d = '' == false
let e = '0' == 0
"
  a == false
  b == false
  c == false
  d == false
  e == false
);

bang_test!(not_equal_numbers
"
let a = 3 != 4
let b = 3 != 3
let c = -5 != 5
"
  a == true
  b == false
  c == true
);

bang_test!(not_equal_strings
"
let a = 'hello' != \"hello\"
let b = 'hello' != \"goodbye\"
let c = 'hello' != 'hello!'
let d = \"hello\" != \"hello\"
"
  a == false
  b == true
  c == true
  d == false
);

bang_test!(not_equal_boolean
"
let a = true != true
let b = true != false
let c = false != false
"
  a == false
  b == true
  c == false
);

bang_test!(not_equal_null
"
let a = null != null
"
  a == false
);

bang_test!(not_equal_between_types
"
let a = null != false
let b = 0 != false
let c = null != 0
let d = '' != false
let e = '0' != 0
"
  a == true
  b == true
  c == true
  d == true
  e == true
);

bang_test!(function_equal_to_itself_but_not_identical_definition
"
let a = () => 7

let b = () -> number
  return 7

let c = a == a
let d = a == b
let e = a() == a()
let f = a() == b()
let g = print == print
let h = print == type
"
  c == true
  d == false
  e == true
  f == true
  g == true
  h == false
);

bang_test!(built_strings
"
let a = 'hel' + 'lo' == 'hello'
let b = 'hell' + 'o' == 'he' + 'llo'
"
  a == true
  b == true
);

bang_test!(unterminated_string
"
'hello"
  CompileError
);

bang_test!(unterminated_string_2
"
`hello
"
  CompileError
);

bang_test!(lists_equality
"
let a = [] == []
let b = [1, 2, 3] == [1, 2, 3]
let c = [1, 2, 3] == [1, 2, 3, 4]
let d = [1, 2, 3] == [1, 2]
let e = [1, [2, 3]] == [1, [2, 3]]

let f = [1, 'hello', null] == [1, 'hello', null]
let g = [1, 'hello', null]
let h = g == g
let i = [1, 'hello', null] == g
"
  a == true
  b == true
  c == false
  d == false
  e == true
  f == true
  h == true
  i == true
);
