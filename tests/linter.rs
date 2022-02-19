use bang;

macro_rules! bang_lint {
  ($name:ident $code:literal $($rule:literal [$($num:literal)*])*) => {
    #[test]
    fn $name() {
      let tokens = bang::tokenize($code);
      match bang::parse(&tokens) {
        Ok(ast) => {
          let warnings = bang::lint(&ast);
          $({
            let warning = warnings.iter().find(|warning| warning.title == $rule).unwrap();
            assert_eq!(warning.lines, vec![$($num),*]);
          };)*
        }
        Err(_) => assert!(false, "Failed to parse code"),
      }
    }
  };
}

bang_lint!(no_constant_condition
"
if (true)
  do_stuff()
else
  do_stuff()
if (x > 6)
  do_stuff()
else if (false)
  do_stuff()
while (4 > 5)
  do_stuff()
while (question() > 4)
  do_stuff()
"
  "No Constant Conditions" [2 8 10]
);

bang_lint!(no_negative_zero
"
-0
0
--0
-0.0
"
  "No Negative Zero" [2 4 5]
);

bang_lint!(no_self_assign
"
let a = 8
  let a = a
  a += 1
let b = 0
b = b
"
  "No Self Assign" [6]
);

bang_lint!(no_unreachable_code
"
let x = () ->
  if (true)
    return 10
    type(\"Hello\")
  else
    return 5
let y = () ->
  if (true)
    return 10
  x()
  return
  (77)
  null
"
  "No Unreachable Code" [4 12]
);

bang_lint!(no_yoda_equality_check
"
x == y
x == 7
7 == x
x != y
x != 7
7 != x
7 < x
x > 7
"
  "No Yoda Equality" [4 7]
);

bang_lint!(no_constant_condition_and_unreachable_code
"
if (true)
  do_stuff()
  return 10
  do_other_stuff()
else
  do_stuff()

let a = 7
if (-(a = 8))
  do_stuff()
"
  "No Constant Conditions" [2 10]
  "No Unreachable Code" [4]
);
