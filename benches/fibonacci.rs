#![feature(test)]

mod bang_benchmark;
use bang_benchmark::*;

bang_benchmark!(
  recursive_fibonacci,
  "
fun fib_recursive(n: number)
  if (n <= 2)
    if (n == 0)
      return 0
    return n - 1
  else
    return fib_recursive(n - 1) + fib_recursive(n - 2)

fib_recursive(25)
"
);

bang_benchmark!(
  iterative_fibonacci,
  "
fun fib_iterative(n: number)
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
"
);
