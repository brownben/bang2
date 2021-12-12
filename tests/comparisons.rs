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

bang_test!(equality_number
"
let a = 3 == 4
let b = 3 == 3
let c = -5 == 5
"
 a == false
 b == true
 c == false
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
