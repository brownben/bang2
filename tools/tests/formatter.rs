use bang_syntax::parse;
use bang_tools::format;

macro_rules! assert_format {
  ($source:expr) => {
    assert_format!($source, $source)
  };

  ($source:expr, $output:expr) => {
    let ast = parse($source).unwrap();
    let formatter = format($source, &ast);
    let output = formatter.to_string();

    assert_eq!(output.trim(), $output.trim());
    assert_format!(remains_constant output);
  };

  (remains_constant $source:expr) => {
    let ast = parse(&$source).unwrap();
    let formatter = format(&$source, &ast);

    assert_eq!(&formatter.to_string(), &$source);
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
  assert_format!("3 / 4");
  assert_format!("3 > 4");
  assert_format!("3 < 4");
  assert_format!("3 == 4");
  assert_format!("3 != 4");
  assert_format!("3 >= 4");
  assert_format!("3 <= 4");
  assert_format!("null ?? 4");
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
  assert_format!("(a: number) => null", "(a: number) => null");
  assert_format!("(  a : string )   =>null", "(a: string) => null");
}

#[test]
fn function_with_parameters_multiline() {
  assert_format!(
    "(a: number,b   : string,) => null",
    "(a: number, b: string) => null"
  );
  assert_format!(
    "(\na: c, b: d   )    =>    null",
    "(\n  a: c, b: d, \n) => null"
  );
  assert_format!(
    "(a: a, \nb: b, \n c: c) => null",
    "(\n  a: a,\n  b: b,\n  c: c,\n) => null"
  );
}

#[test]
fn declaration_statement() {
  assert_format!("let x = 7", "let x = 7");
  assert_format!("let    x=7", "let x = 7");
  assert_format!("let print = ( ) =>  null", "let print = () => null");
  assert_format!("let print");
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
  assert_format!("x -= 1", "x -= 1");
  assert_format!("x = x - 1", "x -= 1");
  assert_format!("x *= 1", "x *= 1");
  assert_format!("x = x * 1", "x *= 1");
  assert_format!("x /= 1", "x /= 1");
  assert_format!("x = x / 1", "x /= 1");
}

#[test]
fn fibonacci_iterative() {
  let fibonacci_iterative = "
// Iterative Implementation of Fibonacci

let fib_iterative = (n: number) -> number
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
let block = (n: null) ->
  if (true)
    stuff
    more

block(25)
    ";
  assert_format!(source);

  let source = "
let block = (n: null) ->
  if (true)
    stuff
    more
  other

block(25)
    ";
  assert_format!(source);

  let source = "
let block = (n: null) ->
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
let block = (n: null) ->
  if (true) stuff
  else other

block(25)
    ";
  assert_format!(source);

  let source = "
let block = (n: null) ->
  if (true)
    let a = [
      2,
      4,
      6,
    ]
    return a
  else
    let a = [
      1,
      3,
      5,
    ]
    return a
block(25)
    ";
  assert_format!(source);
}

#[test]
fn fibonacci_recursive() {
  let fibonacci_recursive = "
let fib_recursive = (n: number) -> number
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
  assert_format!("from 'maths' import { sqrt }", "from maths import { sqrt }");
  assert_format!(
    "from './maths' import { sqrt }",
    "from './maths' import { sqrt }"
  );
  assert_format!(
    "from `./maths` import { sqrt }",
    "from './maths' import { sqrt }"
  );
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

#[test]
fn declaration_statement_with_type() {
  assert_format!("let x :   number = 7", "let x: number = 7");
  assert_format!("let    x:null|number   =7", "let x: null | number = 7");
  assert_format!("let print: null", "let print: null");
  assert_format!(
    "let print: null|function|boolean",
    "let print: null | function | boolean"
  );
}

#[test]
fn list() {
  assert_format!("[1, 2, 3]");
  assert_format!("[1, 2, 3,]", "[1, 2, 3]");
  assert_format!("[]");
  assert_format!("[  ] ", "[]");
  assert_format!("['hello',    () =>   8, 5]", "['hello', () => 8, 5]");
  assert_format!("[\n  'hello', () => 8, 5, \n]");
  assert_format!("[\n  'hello',\n  () => 8,\n  5,\n]");
}

#[test]
fn types() {
  assert_format!("let a: number?");
  assert_format!("let b: number[]");
  assert_format!("let c: string | number");
  assert_format!("let d: (string | number)[]");
  assert_format!("let e: (string) -> number");
  assert_format!("let e: (string,) -> number", "let e: (string) -> number");
  assert_format!("let f: (string, number) -> number");
  assert_format!(
    "let f: < T, D > (string, number) -> number",
    "let f: <T, D>(string, number) -> number"
  );
}

#[test]
fn pipeline() {
  assert_format!("7 >> multiply(4)");
}

#[test]
fn index() {
  assert_format!("'hello' [ 3 ] ", "'hello'[3]");
  assert_format!("'hello' [ 3 ] [ 4 ] ", "'hello'[3][4]");
  assert_format!("'hello' [ 3 + 4] ", "'hello'[3 + 4]");
  assert_format!("'hello' [\n3 + 4\n] [ \n5 ] ", "'hello'[3 + 4][5]");
}

#[test]
fn index_assignment() {
  assert_format!("x[3] += 7");
  assert_format!("x[3 - 8] = 22");
  assert_format!("[1, 2, 3][0] /= 99");
}

#[test]
fn list_destructuring() {
  assert_format!("let[a,b]=[1,2,3]", "let [a, b] = [1, 2, 3]");
  assert_format!("let [c,] =  list", "let [c] = list");
}

#[test]
fn format_string() {
  assert_format!("'hello ${7}'");
  assert_format!("'hello ${   7 }'", "'hello ${7}'");
  assert_format!("'hello ${3 + 1} world'");
  assert_format!("'${7} world'");
  assert_format!("call('${7} world')");
  assert_format!("'hello ${7} world ${false}!'");
}

#[test]
fn module_access() {
  assert_format!("maths::sin(7)");
  assert_format!("maths :: sin ( 6 )", "maths::sin(6)");
}

#[test]
fn line_spacing() {
  assert_format!(
    "
from list import { map }
from maths import { sin }
from list import { map as mapTwo }

let a = map == map
let b = map == list::map
let c = sin == sin
let d = map == mapTwo
let e = sin == maths::sin

print([a, b, c, d, e])
  "
  );

  assert_format!(
    "
let a = () ->
  from list import { map }
  from maths import { sin }
  from list import { map as mapTwo }

  let a = map == map
  let b = map == list::map
  let c = sin == sin
  let d = map == mapTwo
  let e = sin == maths::sin

  print([a, b, c, d, e])
  "
  );
  assert_format!(
    "
let result = 0
let i = 0
while (i < 100000)
  result += 11
  result *= 10
  result -= (result / 100) * 99
  i += 1
    "
  );
  assert_format!(
    "
let bubbleSort = (list) -> null
  let n = length(list)
  let i = 0

  while (i < n)
    let j = 0

    while (j < n - i - 1)
      if (list[j] > list[j + 1]) list >> swap(j, j + 1)
      j += 1

    i += 1
"
  );
}
