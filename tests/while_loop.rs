mod bang_test;
use bang_test::*;

bang_test!(while_loop
"
let x = 0
while (x < 10)
  x += 1
"
  x == 10.0
);

bang_test!(while_false
"
let x = 0
while (false)
  x += 1
"
  x == 0.0
);
