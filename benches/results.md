# Bang

```
test arithmetic::all         ... bench:  77,037,940 ns/iter (+/- 10,190,124)
test arithmetic::to_bytecode ... bench:      16,576 ns/iter (+/- 2,955)
test arithmetic::vm          ... bench:  81,299,840 ns/iter (+/- 10,922,057)
test iterative_fibonacci::all         ... bench:      24,946 ns/iter (+/- 6,628)
test iterative_fibonacci::to_bytecode ... bench:      14,855 ns/iter (+/- 8,909)
test iterative_fibonacci::vm          ... bench:       8,311 ns/iter (+/- 4,815)
test recursive_fibonacci::all         ... bench:  45,483,720 ns/iter (+/- 20,199,311)
test recursive_fibonacci::to_bytecode ... bench:      14,211 ns/iter (+/- 3,019)
test recursive_fibonacci::vm          ... bench:  42,817,290 ns/iter (+/- 4,658,076)

// New Hash Table
test arithmetic::all         ... bench:  66,182,230 ns/iter (+/- 9,109,979)
test arithmetic::to_bytecode ... bench:      11,290 ns/iter (+/- 291)
test arithmetic::vm          ... bench:  66,161,680 ns/iter (+/- 14,211,344)
test iterative_fibonacci::all         ... bench:      20,471 ns/iter (+/- 6,432)
test iterative_fibonacci::to_bytecode ... bench:      13,300 ns/iter (+/- 3,888)
test iterative_fibonacci::vm          ... bench:       7,484 ns/iter (+/- 1,688)
test recursive_fibonacci::all         ... bench:  46,191,580 ns/iter (+/- 16,715,700)
test recursive_fibonacci::to_bytecode ... bench:      16,694 ns/iter (+/- 4,380)
test recursive_fibonacci::vm          ... bench:  41,410,330 ns/iter (+/- 12,002,097)

// New parser + hash table
test arithmetic::all         ... bench:  78,505,870 ns/iter (+/- 8,709,342)
test arithmetic::to_bytecode ... bench:      10,247 ns/iter (+/- 1,186)
test arithmetic::vm          ... bench:  76,894,680 ns/iter (+/- 12,186,257)
test iterative_fibonacci::all         ... bench:      16,582 ns/iter (+/- 4,297)
test iterative_fibonacci::to_bytecode ... bench:      10,002 ns/iter (+/- 1,597)
test iterative_fibonacci::vm          ... bench:       6,756 ns/iter (+/- 2,687)
test recursive_fibonacci::all         ... bench:  39,040,240 ns/iter (+/- 5,994,092)
test recursive_fibonacci::to_bytecode ... bench:      10,975 ns/iter (+/- 560)
test recursive_fibonacci::vm          ... bench:  42,165,520 ns/iter (+/- 10,747,315)
```

# Python

```
Arithmetic: 2,706,981,700 ns
Iterative Fibonacci: 300,699 ns
Recursive Fibonacci: 2,201,753,900 ns
```

# Node

```
Arithmetic 4,399,700 ns
Fib Iterative 78,000 ns
Fib Recursive 2,297,799 ns
```
