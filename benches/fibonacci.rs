#![feature(test)]

mod bang_benchmark;
use bang_benchmark::*;

bang_benchmark!(
  recursive_fibonacci,
  "
let fibonacci_recursive = (n: number) -> number
  if n < 2
    return n
  else
    return fibonacci_recursive(n - 1) + fibonacci_recursive(n - 2);

fib_recursive(25)
"
);

bang_benchmark!(
  iterative_fibonacci,
  "
let fib_iterative(n: number) -> number
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
