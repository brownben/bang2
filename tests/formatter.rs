use bang::{format, parse, tokenize};

macro_rules! assert_format {
  ($source:expr) => {
    assert_format!($source, $source)
  };
  ($source:expr, $output:expr) => {
    let tokens = tokenize($source);
    let ast = parse($source, &tokens).unwrap();
    let formatter = format($source, &ast);

    assert_eq!(formatter.to_string().trim(), $output.trim());
  };
}

#[test]
fn blank_program_outputs_nothing() {
  assert_format!("", "");
}

#[test]
fn binary_expression() {
  assert_format!("3+4", "3 + 4");
  assert_format!("3+4", "3 + 4");
  assert_format!("3 -4", "3 - 4");
  assert_format!("3* 4", "3 * 4");
}

#[test]
fn unary_expression() {
  assert_format!("! true", "!true");
  assert_format!("-5.2", "-5.2");
}

#[test]
fn grouping() {
  assert_format!("(5 or false)", "(5 or false)");
  assert_format!("( 5||  false ) ", "(5 or false)");
  assert_format!("( 5||  false ) and true", "(5 or false) and true");
}

#[test]
fn multiline_grouping() {
  assert_format!(
    "
(
  multiline(
    1,
    2,
    3,
  )
)"
  );
  assert_format!(
    "
(
  multiline(
    1,
    2,
    3,
  )

)",
    "
(
  multiline(
    1,
    2,
    3,
  )
)"
  );
}

#[test]
fn and_or_prefer_words() {
  assert_format!("3 and 4", "3 and 4");
  assert_format!("3 or   4", "3 or 4");
  assert_format!("3 || 4", "3 or 4");
  assert_format!("3   && 4", "3 and 4");
  assert_format!("3||4", "3 or 4");
  assert_format!("3&&4", "3 and 4");
}

#[test]
fn strings_prefer_single_quotes() {
  assert_format!("'hello'", "'hello'");
  assert_format!("`world`", "'world'");
  assert_format!("\"from bang\"", "'from bang'");
}

#[test]
fn numbers_are_simplified_unless_numeric_separators() {
  assert_format!("3.00", "3");
  assert_format!("3.01", "3.01");
  assert_format!("000", "0");
  assert_format!("0001", "1");
  assert_format!("0.1", "0.1");

  assert_format!("1_000_000", "1_000_000");
  assert_format!("000_000.1", "000_000.1");
}

#[test]
fn call_expression_no_args() {
  assert_format!("print (  )", "print()");
  assert_format!("doStuff(  )", "doStuff()");
  assert_format!("aGreatFunction()", "aGreatFunction()");
}

#[test]
fn call_expression_one_args() {
  assert_format!("print ( 1 )", "print(1)");
  assert_format!("doStuff(  false,)", "doStuff(false)");
  assert_format!("aGreatFunction(object)", "aGreatFunction(object)");
}

#[test]
fn call_all_same_line() {
  assert_format!("print ( 1, 3  ,2 )", "print(1, 3, 2)");
  assert_format!("doStuff(  false, null)", "doStuff(false, null)");
  assert_format!("aGreatFunction(object,)", "aGreatFunction(object)");
}

#[test]
fn call_all_one_line() {
  assert_format!("print(\n  1, 3, 2, \n)", "print(\n  1, 3, 2, \n)");
  assert_format!("print  (\n  1  ,  3,2, \n  )", "print(\n  1, 3, 2, \n)");
}

#[test]
fn call_split_lines() {
  assert_format!("print(\n  1, \n 3, 2,\n)", "print(\n  1,\n  3,\n  2,\n)");
  assert_format!(
    "print  ( 1  ,  3, \n2, \n  )",
    "print(\n  1,\n  3,\n  2,\n)"
  );
  assert_format!(
    "print(\n  1,\n  3,\n   2,\n)",
    "print(\n  1,\n  3,\n  2,\n)"
  );
}

#[test]
fn call_nested_split_lines() {
  assert_format!(
    "print(\n  1.0,\n  print( ),2,\n)",
    "print(\n  1,\n  print(),\n  2,\n)"
  );
  assert_format!(
    "print(\n  1,\n  print(\n1, \n   2),\n  2,\n)",
    "print(\n  1,\n  print(\n    1,\n    2,\n  ),\n  2,\n)"
  );
}

#[test]
fn comment_expression() {
  assert_format!("x//   comment", "x // comment");
  assert_format!(
    "doStuff(  false,)    //    A  wEiRd-MeSsAgE",
    "doStuff(false) // A  wEiRd-MeSsAgE"
  );
}

#[test]
fn asssignment_expression() {
  assert_format!("x = 6", "x = 6");
  assert_format!("x    =   `string`   ", "x = 'string'");
}

#[test]
fn function_with_expression_body() {
  assert_format!("() => null", "() => null");
  assert_format!("()    =>    null", "() => null");
  assert_format!("(a) => null", "(a) => null");
  assert_format!("(  a )   =>null", "(a) => null");
}

#[test]
fn function_with_parameters_multiline() {
  assert_format!("(a,b,) => null", "(a, b) => null");
  assert_format!("(\na, b)    =>    null", "(\n  a, b, \n) => null");
  assert_format!("(a, \nb, \n c) => null", "(\n  a,\n  b,\n  c,\n) => null");
}

#[test]
fn declaration_statement() {
  assert_format!("let x = 7", "let x = 7");
  assert_format!("let    x=7", "let x = 7");
  assert_format!("let print = ( ) =>  null", "let print = () => null");
}

#[test]
fn comment_statement() {
  assert_format!("//   whats up", "// whats up");
}

#[test]
fn return_statement() {
  assert_format!("return null", "return null");
  assert_format!("return\n", "return");
  assert_format!("return     3+  5", "return 3 + 5");
}

#[test]
fn if_statement() {
  assert_format!("if (true) doStuff()", "if (true) doStuff()");
  assert_format!("if(  true )doStuff()", "if (true) doStuff()");
  assert_format!(
    "if (true)\n  doStuff()\nelse\n  doOtherStuff()",
    "if (true) doStuff()\nelse doOtherStuff()"
  );
  assert_format!(
    "if (true) (1 + 2)\nelse false",
    "if (true) (1 + 2)\nelse false"
  );
}

#[test]
fn if_else_statement() {
  assert_format!(
    "
if (true)
  // another statment
  doStuff()
else
  doOtherStuff()
",
    "
if (true)
  // another statment
  doStuff()
else doOtherStuff()"
  );
  assert_format!(
    "
if (true)
  // another statment
  doStuff()
else
  moreStuff()
  doOtherStuff()
",
    "
if (true)
  // another statment
  doStuff()
else
  moreStuff()
  doOtherStuff()"
  );
}

#[test]
fn if_statement_multiline_condition() {
  assert_format!(
    "if (function(1, \n2,\n) == false) doStuff()",
    "if (\n  function(\n    1,\n    2,\n  ) == false\n) doStuff()"
  );
  assert_format!(
    "if (function(1, \n2,\n) == false)\n  // comment\n  doStuff()",
    "if (\n  function(\n    1,\n    2,\n  ) == false\n)\n  // comment\n  doStuff()"
  );
}

#[test]
fn else_if_statement() {
  assert_format!(
    "if (true) doStuff()\nelse if (false) dontDo()\nelse doOtherStuff()",
    "if (true) doStuff()\nelse if (false) dontDo()\nelse doOtherStuff()"
  );
}

#[test]
fn while_statement() {
  assert_format!("while (true) doStuff()", "while (true) doStuff()");
  assert_format!("while(  true )doStuff()", "while (true) doStuff()");
  assert_format!("while (true)\n  doStuff()", "while (true) doStuff()");
  assert_format!(
    "while (true)\n  (1 + 2)\n  false",
    "while (true)\n  (1 + 2)\n  false"
  );
}

#[test]
fn assignment_operator() {
  assert_format!("x += 1", "x += 1");
  assert_format!("x = x + 1", "x += 1");
}

#[test]
fn fibonacci_iterative() {
  let fibonacci_iterative = "
// Iterative Implementation of Fibonacci

let fib_iterative = (n) ->
  let x = 0
  let y = 1
  let i = 1
  while (i < n)
    let z = x + y
    x = y
    y = z
    i += 1
  return x

fib_iterative(25)
";
  assert_format!(fibonacci_iterative);
}

#[test]
fn if_statement_in_block() {
  let source = "
let block = (n) ->
  if (true)
    stuff
    more

block(25)
    ";
  assert_format!(source);

  let source = "
let block = (n) ->
  if (true)
    stuff
    more
  other

block(25)
    ";
  assert_format!(source);

  let source = "
let block = (n) ->
  if (true)
    stuff
    more
  else
    other
    moreOther

block(25)
";
  assert_format!(source);

  let source = "
let block = (n) ->
  if (true) stuff
  else other

block(25)
    ";
  assert_format!(source);
}

#[test]
fn fibonacci_recursive() {
  let fibonacci_recursive = "
let fib_recursive = (n) ->
if (n <= 2)
  if (n == 0) return 0
  return n - 1
else return fib_recursive(n - 1) + fib_recursive(n - 2)

fib_recursive(25)";

  assert_format!(fibonacci_recursive);
}

#[test]
fn import_statement() {
  assert_format!("from maths import { sqrt }");
  assert_format!("from maths import { sqrt, pow }");
  assert_format!("from maths import { sqrt, pow, abs }");
  assert_format!("from maths import {\n  sqrt, pow, abs, floor, \n}");
  assert_format!("from maths import {\n  sqrt,\n  pow,\n  abs,\n  floor,\n}");
}

#[test]
fn import_statement_alias() {
  assert_format!("from maths import { sqrt as squareRoot }");
  assert_format!("from maths import { sqrt, pow as power }");
  assert_format!("from maths import { sqrt, pow, abs }");
  assert_format!("from maths import {\n  sqrt, pow, abs as absolute, floor, \n}");
  assert_format!(
    "from maths import {\n  sqrt as squareRoot,\n  pow as power,\n  abs,\n  floor,\n}"
  );
}
