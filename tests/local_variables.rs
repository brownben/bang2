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
