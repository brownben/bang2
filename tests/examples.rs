mod bang_test;
use bang_test::*;

bang_test!(arithmetic
"
let result = 0
let i = 0
while(i < 100000)
    result += 11
    result *= 10
    result -= (result / 100) * 99
    i += 1
"
  i == 100000.0
);
