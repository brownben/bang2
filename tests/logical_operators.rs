mod bang_test;
use bang_test::*;

bang_test!(not
"
let a = !null
let b = !\"\"
let c = !true
let d = !false
let e = !\"Hello\"
let f = !0
let g = !-1
let h = !3
"
  a == true
  b == true
  c == false
  d == true
  e == false
  f == true
  g == false
  h == false
);

bang_test!(and
"
let a = false && true
let b = false && false
let c = true && true
let d = true && false
let e = null && \"Hello\"
let f = true && \"Hello\"

let eIsNull = e == null
"
  a == false
  b == false
  c == true
  d == false
  eIsNull == true
  f == "Hello"
);

bang_test!(or
"
let a = false || true
let b = false || false
let c = true || true
let d = true || false
let e = null || \"Hello\"
let f = true || \"Hello\"
"
  a == true
  b == false
  c == true
  d == true
  e == "Hello"
  f == true
);

bang_test!(and_word
"
let a = false and true
let b = false and false
let c = true and true
let d = true and false
let e = null and \"Hello\"
let f = true and \"Hello\"

let eIsNull = e == null
"
  a == false
  b == false
  c == true
  d == false
  eIsNull == true
  f == "Hello"
);

bang_test!(or_word
"
let a = false or true
let b = false or false
let c = true or true
let d = true or false
let e = null or \"Hello\"
let f = true or \"Hello\"
"
  a == true
  b == false
  c == true
  d == true
  e == "Hello"
  f == true
);

bang_test!(nullish_coalescing
"
let a = false ?? true
let b = null ?? false
let c = true ?? false
let d = null ?? \"Hello\"
let e = true ?? \"Hello\"
let f = 0 ?? \"Hello\"
"
  a == false
  b == false
  c == true
  d == "Hello"
  e == true
  f == 0.0
);

bang_test!(and_shortcircuit
"
let a = \"before\"
let b = \"before\"
(a = true) and (b = false) and
  (a = \"bad\")
"
  a == true
  b == false
);

bang_test!(or_shortcircuit
"
let a = \"before\"
let b = \"before\"
(a = false) or (b = true) or (a = \"bad\")
"
  a == false
  b == true
);
