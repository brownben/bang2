#![feature(test)]

mod bang_benchmark;
use bang_benchmark::*;

bang_benchmark!(
  arithmetic,
  "
let result = 0
let i = 0
while(i < 100000)
    result += 11
    result *= 10
    result -= (result / 100) * 99
    i += 1
"
);
