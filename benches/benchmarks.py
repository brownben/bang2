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


def loop():
    i = 0
    while i < 100000:
        i = i + 1
        1
        1
        1
        2
        1
        None
        1
        "str"
        1
        True
        None
        None
        None
        1
        None
        "str"
        None
        True
        True
        True
        True
        1
        True
        False
        True
        "str"
        True
        None
        "str"
        "str"
        "str"
        "stru"
        "str"
        1
        "str"
        None
        "str"
        True


def equality():
    i = 0
    while i < 100000:
        i = i + 1
        1 == 1
        1 == 2
        1 == None
        1 == "str"
        1 == True
        None == None
        None == 1
        None == "str"
        None == True
        True == True
        True == 1
        True == False
        True == "str"
        True == None
        "str" == "str"
        "str" == "stru"
        "str" == 1
        "str" == None
        "str" == True


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

result = timeit.timeit(lambda: loop(), number=10)
print(f"Loop: {result}s")

result = timeit.timeit(lambda: equality(), number=10)
print(f"Equality: {result}s")

result = timeit.timeit(lambda: arithmetic(), number=10)
print(f"Arithmetic: {result}s")
