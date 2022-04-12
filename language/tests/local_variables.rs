mod bang_test;
use bang_test::*;

bang_test!(can_assign_local_variables
"
let global
  let local = 7
  global = local
"
  global == 7.0
);

bang_test!(can_use_local_variables
"
let global
  let a = 2
  let b = 3
  let c = a + b
  global = c / 2
"
  global == 2.5
);

bang_test!(shadows_higher_scopes
"
let a
let b

let hello = 0
if (hello == 0)
  let hello = 1
  a = hello + 1

b = hello
"
  a == 2.0
  b == 0.0
);

bang_test!(define_variables_twice
"
1 + 1
  let hello = 0
  let hello = 1
"
  CompileError
);

bang_test!(initialised_to_null
"
let global = 9
  let a
  global = a

let globalIsNull = global == null
"
  globalIsNull == true
);

bang_test!(missing_variable_name
"
1 + 1
  let = 0
"
  CompileError
);

bang_test!(unknown_global
"
let a = unknownGlobal
"
  RuntimeError
);

bang_test!(unknown_global_set
"
unknownGlobal = 8
"
  RuntimeError
);

bang_test!(cant_assign_to_expression
"
let a
a + 1 = 8
"
  CompileError
);

bang_test!(cant_assign_to_literal
"
7 = 8
"
  CompileError
);

bang_test!(hanging_equals
"
7 =
"
  CompileError
);

bang_test!(assignment
"
let a
  let b
  b = 22
  a = b
"
  a == 22.0
);

bang_test!(multiple_nested
"
let a
  let b
    b = 22
  a = b
"
  a == 22.0

);
