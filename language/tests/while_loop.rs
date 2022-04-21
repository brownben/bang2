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

bang_test!(arithmetic
"
let result = 0
let i = 0
while(i < 100)
    result += 11
    result *= 10
    result -= (result / 100) * 99
    i += 1
"
  i == 100.0
);

bang_test!(local_variable_loops
"
let loopy = (n: number) ->
  let i = 0
  while (i < n)
    if (i < 3)
      let temp = i + 1
      temp *= 2
    i += 1
  return i

let x = loopy(9)
"
  x == 9
);

bang_test!(local_variable_loops_nested
"
let loopy = (n: number) ->
  let i = 0
  while (i < n)
    let j = 0
    while (j < n)
      if (i < j)
        let temp = i + 1
        temp *= 2
      j += 1
    i += 1
  return i

let x = loopy(9)
"
  x == 9
);
