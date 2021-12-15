#![feature(test)]

mod bang_benchmark;
use bang_benchmark::*;

bang_benchmark!(
  equality_loop,
  "
let i = 0
while (i < 100000)
  i = i + 1
  1
  1
  1
  2
  1
  null
  1
  'str'
  1
  true
  null
  null
  null
  1
  null
  'str'
  null
  true
  true
  true
  true
  1
  true
  false
  true
  'str'
  true
  null
  'str'
  'str'
  'str'
  'stru'
  'str'
  1
  'str'
  null
  'str'
  true
"
);

bang_benchmark!(
  equality,
  "
let i = 0
while (i < 100000)
  i += 1
  1 == i
  1 == 2
  1 == null
  1 == 'str'
  1 == true
  null == null
  null == 1
  null == 'str'
  null == true
  true == true
  true == 1
  true == false
  true == 'str'
  true == null
  'str' == 'str'
  'str' == 'stru'
  'str' == 1
  'str' == null
  'str' == true
"
);
