const { PerformanceObserver, performance } = require('perf_hooks')

const obs = new PerformanceObserver((items) => {
  for (const entry of items.getEntries()) {
    console.log(entry.name, entry.duration, 'ms')
  }

  performance.clearMarks()
})
obs.observe({ type: 'measure' })

function fib_recursive(n) {
  if (n <= 2) {
    if (n == 0) return 0
    return n - 1
  }
  return fib_recursive(n - 1) + fib_recursive(n - 2)
}

function fib_iterative(n) {
  let x = 0
  let y = 1
  let i = 1
  while (i < n) {
    z = x + y
    x = y
    y = z
    i += 1
  }
  return x
}

function arithmetic() {
  let result = 0
  let i = 0
  while (i < 100000) {
    result += 11
    result *= 10
    result -= (result / 100) * 99
    i += 1
  }
}

performance.mark('C')
arithmetic()
performance.measure('Arithmetic', 'C')

performance.mark('B')
fib_iterative(25)
performance.measure('Fib Iterative', 'B')

performance.mark('A')
fib_recursive(25)
performance.measure('Fib Recursive', 'A')
