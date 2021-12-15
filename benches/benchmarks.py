import timeit


def fib_recursive(n):
    if n <= 2:
        if n == 0:
            return 0
        return n - 1
    else:
        return fib_recursive(n - 1) + fib_recursive(n - 2)


def fib_iterative(n):
    x = 0
    y = 1
    i = 1
    while i < n:
        z = x + y
        x = y
        y = z
        i += 1
    return x


def arithmetic():
    result = 0
    i = 0
    while i < 100000:
        result += 11
        result *= 10
        result -= (result / 100) * 99
        i += 1


result = timeit.timeit(lambda: fib_recursive(25), number=10)
print(f"Recursive Fibonacci: {result}s")


result = timeit.timeit(lambda: fib_iterative(25), number=10)
print(f"Iterative Fibonacci: {result}s")


result = timeit.timeit(lambda: arithmetic(), number=10)
print(f"Arithmetic: {result}s")
