mod bang_test;
use bang_test::*;

bang_test!(comments_start_blocks
"
let a = type(3)
if (true)
  // comment
  a = type(true)
else
  // another comment
  a = type(1)
"
  a == "boolean"
);

bang_test!(comments_in_blocks
"
let a = type(3)
if (true)
  a = type(true)
  // comment
  type(a)
else
  a = type(1)
  // another comment
  let a = 7
"
  a == "boolean"
);

bang_test!(comments_end_blocks
"
let a = type(3)
if (true)
  a = type(true)
  // comment
else
  a = type(1)
  // another comment
"
  a == "boolean"
);

bang_test!(comments_end_blocks_2
"
let a = type(3)
if (true)
  a = type(true)
  // comment
  // another one
else
  a = type(1)
  // another comment

while (false)
  let a = 7
  // nothing
"
  a == "boolean"
);

bang_test!(comments_at_end_of_line
"
let a = type(3) // hello
if (true)
  4 + 5 // hello
  a = type(true)
else
  a = type(1)
"
  a == "boolean"
);
