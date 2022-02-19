mod bang_test;
use bang_test::*;

bang_test!(comments_in_blocks_1
"
let a = type(3)
if (true)
  // comment
  a = type(true)
else
  a = type(1)
"
  a == "boolean"
);

bang_test!(comments_at_end
"
let a = type(3) // hello
if (true)
  4 + 5 // hello
  a = type(true)
else // hello
  a = type(1)
"
  a == "boolean"
);
