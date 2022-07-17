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

bang_test!(close_multiple_scopes_at_once
"
let a
if (true)
  if (true)
    a = true

let b
if (true)
  if (true)
    b = true
"
  a == true
  b == true
);

bang_test!(function_is_truthy
"
let a
if (print)
    a = true

let identity = (x: any) => x

let b
if (identity)
  b = true
"
  a == true
  b == true
);

bang_test!(if_statement_local_variables
"
let iffy = (n: number) ->
  let i = 0
  if (i < 3)
    let temp = i + 1
    temp *= 2

  return i

let x = iffy(9)
"
  x == 0
);

bang_test!(if_else_statement_local_variables
"
let iffy = (n: number) ->
  let i = 0
  if (i < 3)
    let temp = i + 1
    temp *= 2
  else
    let temp = i + 1
    temp *= 4
  return i

let x = iffy(9)
"
  x == 0
);

bang_test!(filled_list_truthy
"
let a
if ([1, 2])
    a = true
"
  a == true
);

bang_test!(empty_list_falsy
"
let a = false
if ([])
    a = true
"
  a == false
);

bang_test!(misformed_if
"
if [])
  doStuff()
"
  CompileError
);

bang_test!(misformed_if_end
"
if ([]
  doStuff()
"
  CompileError
);
