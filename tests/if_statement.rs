mod bang_test;
use bang_test::*;

bang_test!(if_statement_same_line
"
let x
let y
if (true) x = 10
if (false) y = 20

let yIsNull = y == null
"
  x == 10.0
  yIsNull == true
);

bang_test!(if_statement_block
"
let x
let y
if (true)
  x = 10
if (false)
  y = 20

let yIsNull = y == null
"
  x == 10.0
  yIsNull == true
);

bang_test!(if_else_statement_same_line
"
let x
let y
if (true) x = 10
else x = 5

if (false) y = 20
else y = 7
"
  x == 10.0
  y == 7.0
);

bang_test!(if_else_statement_block
"
let x
let y
if (true)
  x = 10
else
  x = 5

if (false)
  y = 20
else
  y = 7
"
  x == 10.0
  y == 7.0
);

bang_test!(if_else_if_else_statement
"
let x

if (false)
  x = 10
else if (true)
  x = 7
else
  x = 5
"
  x == 7.0
);

bang_test!(dangling_else
"
let x
if (true)
  if (false) x = \"bad\"
  else x = \"good\"

let y
if (false)
  if (true) y = \"good\"
  else y = \"bad\"

let yIsNull = y == null
"
  x == "good"
  yIsNull == true
);

bang_test!(assignment_in_condition
"
let a = false
let x
if (a = true) x = 7
"
  x == 7.0
  a == true
);