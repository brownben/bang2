mod bang_test;
use bang_test::*;

bang_test!(type_of
"
let a = type(3)
let b = type(true)
let c = type(null)
let d = type(type)
let e = type('hello')
let f = type([])
let g = type(set::new())
let h = type(dict::new())
"
  a == "number"
  b == "boolean"
  c == "null"
  d == "function"
  e == "string"
  f == "list"
  g == "set"
  h == "dict"
);

bang_test!(print
"
let a = print(3)
"
  a == 3
);

bang_test!(correct_number_of_arguments_builtin_function
"
type(1,2)
"
  RuntimeError
);

bang_test!(unknown_module_import
  "from unknown_module import { stuff }"
  RuntimeError
);

bang_test!(unknown_module_value
  "from maths import { stuff }"
  RuntimeError
);

bang_test!(misformed_import
  "from unknown_module { stuff }"
  CompileError
);

bang_test!(misformed_import_start_curly
  "from unknown_module  stuff }"
  CompileError
);

bang_test!(misformed_import_end_curly
  "from unknown_module { stuff "
  CompileError
);

bang_test!(rename_imports_with_as
  "
from maths import { sqrt as squareRoot }
let squareRootIsFunction = type(squareRoot) == 'function'
"
  squareRootIsFunction == true
);

bang_test!(unknown_module_access
  "unknown_module::stuff"
  RuntimeError
);

bang_test!(unknown_module_item
  "maths::item"
  RuntimeError
);

bang_test!(module_access
  "
let a = maths::cos(0)
let b = '${list::push}'
  "
  a == 1
  b == "<function list::push>"
);

bang_test!(equality
"
from list import { map }
from maths import { sin }
from list import { map as mapTwo }

let a = map == map
let b = map == list::map
let c = sin == sin
let d = map == mapTwo
let e = sin == maths::sin
  "
  a == true
  b == true
  c == true
  d == true
  e == true
);

mod maths {
  use super::*;

  bang_test!(constants
"
from maths import { PI, E, INFINITY }

let bigger = INFINITY > 1000000000000000000000000
"
    PI == 3.141592653589793
    E == 2.718281828459045
    bigger == true
  );

  bang_test!(rounding
"
from maths import { ceil, floor, round }

let ceil_a = ceil(1)
let ceil_b = ceil(1.01)
let ceil_c = ceil(1.5)
let ceil_d = ceil(72.3)
let ceil_e = ceil(false) == null

let floor_a = floor(1)
let floor_b = floor(1.01)
let floor_c = floor(1.5)
let floor_d = floor(72.3)
let floor_e = floor(false) == null

let round_a = round(1)
let round_b = round(1.01)
let round_c = round(1.5)
let round_d = round(72.3)
let round_e = round(false) == null
"

    ceil_a == 1
    ceil_b == 2
    ceil_c == 2
    ceil_d == 73
    ceil_e == true

    floor_a == 1
    floor_b == 1
    floor_c == 1
    floor_d == 72
    floor_e == true

    round_a == 1
    round_b == 1
    round_c == 2
    round_d == 72
    round_e == true
  );

  bang_test!(abs_function
"
from maths import {  abs }

let d = abs(1)
let e = abs(-1)
let f = abs(0)
let g = abs(1.1)
"

    d == 1
    e == 1
    f == 0
    g == 1.1
  );

  bang_test!(root_functions
"
from maths import { sqrt, cbrt }

let a = sqrt(4)
let b = cbrt(8)
"

    a == 2
    b == 2
  );

  bang_test!(hyperbolic_trig
"
from maths import { sinh, cosh, tanh, asinh, acosh, atanh }

let a = sinh(0)
let b = cosh(0)
let c = tanh(0)
let d = asinh(0)
let e = acosh(1)
let f = atanh(0)
  "

    a == 0
    b == 1
    c == 0
    d == 0
    e == 0
    f == 0
  );

  bang_test!(trig
    "
from maths import { sin, cos, tan, PI }

let a = sin(0)
let b = cos(0)
let c = tan(0)
let d = sin(PI / 6)

let dSmall = d > 0.49
let dBig = d < 0.51
  "
    a == 0
    b == 1
    c == 0
    dSmall == true
    dBig == true
  );

  bang_test!(inverse_trig
    "
from maths import { asin, acos, atan, isNan }

let a = asin(0)
let b = acos(1)
let c = atan(0)
let d = isNan(asin(55))
  "
    a == 0
    b == 0
    c == 0
    d == true
  );

  bang_test!(power_functions
    "
from maths import { exp, pow }

let a = exp(0)
let b = exp(1)

let c = pow(2, 3)
let d = 4 >> pow(2)
  "
    a == 1
    b == 2.718281828459045
    c == 8
    d == 16
  );

  bang_test!(logarithms
    "
from maths import { log, ln, E }

let a = ln(1)
let b = ln(E)

let c = log(1, 10)
let d = 100 >> log(10)
  "
    a == 0
    b == 1
    c == 0
    d == 2
  );

  bang_test!(angle_unit_conversions
    "
from maths import { degreesToRadians, radiansToDegrees, PI }

let a = degreesToRadians(180)
let b = radiansToDegrees(PI)
  "
    a ==  3.141592653589793
    b == 180
  );
}

mod string {
  use super::*;

  bang_test!(length
"
from string import { length }

let a = length('  hello  ')
let b = length('hello')
let c = length(3) == null
"
    a == 9
    b == 5
    c == true
  );

  bang_test!(trim
"
from string import { trim, trimStart, trimEnd }

let a = trim('  hello  ')
let b = trimStart('  hello  ')
let c = trimEnd('  hello  ')
let d = trim('hello')
let e = trim('hello ')
let f = trim(false) == null
"
    a == "hello"
    b == "hello  "
    c == "  hello"
    d == "hello"
    e == "hello"
    f == true
  );

  bang_test!(repeat
"
from string import { repeat }

let a = repeat('-', 3)
let b = 'ello ' >> repeat(3)
let c = repeat(3, '-') == null
"
    a == "---"
    b == "ello ello ello "
    c == true
  );

  bang_test!(includes
"
from string import { includes, startsWith, endsWith }

let a = includes('hello', 'x')
let b = includes('hello', 'l')
let c = includes(3, '-') == null
let d = 'whats up' >> includes('up')

let e = 'starts' >> startsWith('start')
let f = 'starts' >> endsWith('s')
let g = 'starts' >> startsWith('t')
"
    a == false
    b == true
    c == true
    d == true

    e == true
    f == true
    g == false
  );

  bang_test!(case_change
    "
from string import { toUpperCase, toLowerCase }

let a = toUpperCase('hello')
let b = toLowerCase('HELLO')
let c = toUpperCase('Hello')
let d = toLowerCase('HelLo')
"
    a == "HELLO"
    b == "hello"
    c == "HELLO"
    d == "hello"
  );

  bang_test!(to_number
    "
from string import { toNumber }
from maths import { isNan }

let a = '3.2' >> toNumber
let b = '180' >> toNumber
let c = '-3.2' >> toNumber
let d = '-18y' >> toNumber >> isNan
  "
    a ==  3.2
    b == 180
    c == -3.2
    d == true
  );

  bang_test!(replace
    "
from string import { replace, replaceOne }

let a = 'hello' >> replace('o', 'o world')
let b = 'hello' >> replace('l', 'll')
let c = 'hello' >> replace('q', 'll')

let d = 'hello' >> replaceOne('o', 'o world')
let e = 'hello' >> replaceOne('l', 'll')
let f = 'hello' >> replaceOne('q', 'll')
  "
    a == "hello world"
    b == "hellllo"
    c == "hello"
    d == "hello world"
    e == "helllo"
    f == "hello"
  );

  bang_test!(split
    "
from string import { split }
from list import { length }

let a = 'hello' >> split('')
let b = a >> length()
let [c, d, e, f, g] = a

let [h, i] = 'hello' >> split('l')
let [j] = 'hello' >> split('d')
  "
   b == 5
   c == "h"
   d == "e"
   e == "l"
   f == "l"
   g == "o"
   h == "he"
   i == "o"
   j == "hello"
  );
}

mod list {
  use super::*;

  bang_test!(length
"
from list import { length, isEmpty }

let a = length([1, 2, 3])
let b = length([])
let c = length(3) == null
let d = isEmpty([])
let e = isEmpty([1, 2, 3])
"
    a == 3
    b == 0
    c == true
    d == true
    e == false
  );

  bang_test!(get
"
from list import { get }

let a = [1, 2, 3] >> get(0)
let b = [1, 2, 3] >> get(1)
let c = [1, 2, 3] >> get(2)
let d = ([1, 2, 3] >> get(3)) == null
let e = [1, 2, 3] >> get(-1)
"
    a == 1
    b == 2
    c == 3
    d == true
    e == 3
  );

  bang_test!(push_pop
"
from list import { push, pop, length }

let a = []
a >> push(1)
let b = length(a)
a >> push(88)
let c = length(a)
let d = a >> pop()
let e = length(a)

let list = [1, 2]
list >> push(3) >> push(4)
let f = list >> pop()
let g = list >> pop()
"
    b == 1
    c == 2
    d == 88
    e == 1
    f == 4
    g == 3
  );

  bang_test!(includes
"
from list import { includes }

let a = [] >> includes(7)
let b = [1, 2, 3] >> includes(2)
let c = [1, 2, 3] >> includes(4)
"
    a == false
    b == true
    c == false
  );

  bang_test!(reverse
"
from list import { reverse }

let a = [1, 2, 3]
let b = [1, 2, 3] >> reverse()
let c = b == [3, 2, 1]
let d = a == [1, 2, 3]
"
    c == true
    d == true
  );

  bang_test!(to_set
"
from list import { toSet }

let a = [1, 2, 3]
let b = [1, 2, 3] >> toSet()
let c = type(b)
"
    c == "set"
  );

  bang_test!(hash_pointer
  "
let a = dict::new()
let b = []
a[b] = 0

b >> list::push(3)
a[b] += 1

let x = a[b]
  "
    x == 1
  );

  bang_test!(cyclic_equals
  "
let a = []
let b = [a]
a >> list::push(b)
let x = a == a
let y = a != b
  "
    x == true
    y == true
  );

  bang_test!(cyclic_to_string
  "
let a = []
let b = [a]

a >> list::push(b)

let x = toString(a)
let y = toString(b)

a >> list::push(7)

let w = toString(a)
let z = toString(b)
  "
    x == "[[...]]"
    y == "[[...]]"
    w == "[[...], 7]"
    z == "[[..., 7]]"
  );

  bang_test!(complex_cyclic_to_string
    "
let b = ['b']
let c = ['c', b]
let a = ['a', b]

b >> list::push(c)

let x = toString(a)
let y = toString(b)
"
    x == "['a', ['b', ['c', ...]]]"
    y == "['b', ['c', ...]]"
  );

  bang_test!(complex_cyclic_equals
    "
let b = ['b']
let c = ['c', b]
let a = ['a', b]

b >> list::push(c)

let x = a == a
let y = b == b
let z = a != c
"
    x == true
    y == true
    z == true
  );

  bang_test!(non_cyclic_repeated_to_string
    "
let a = 'a'
let b = [a, a, [a, a]]
let c = ['c', a]
let d = [c, [c, 1, 1]]

let x = toString(b)
let y = toString(d)
"
    x == "['a', 'a', ['a', 'a']]"
    y == "[['c', 'a'], [['c', 'a'], 1, 1]]"
  );

  bang_test!(any_all
  "
from list import { any }

let a = [1, 3, 5, 6] >> any((x) => x > 5)
let b = [1, 3, 5] >> any((x) => x > 5)
let c = [] >> any((x) => x > 5)

let d = [1, 3, 5, 6] >> list::all((x) => x < 5)
let e = [1, 3, 5] >> list::all((x) => x <= 5)
let f = [] >> list::all((x) => x > 5)
  "
    a == true
    b == false
    c == false
    d == false
    e == true
    f == true
  );

  bang_test!(find
  "
from list import { find }

let a = [1, 2, 3] >> find((x) => x >= 2)
let b = [1, 2, 3] >> find((x) => x >= 55)
let c = b == null

  "
    a == 2
    c == true
  );

  bang_test!(for_each
  "
let a = 0
[1, 2, 3] >> list::forEach((x) => a += x)
"
    a == 6
  );

  bang_test!(reduce
  "
let sum = (l) => list::reduce(l, 0, (a, b) => a + b)
let product = (l) => list::reduce(l, 1, (a, b) => a * b)

let a = [1, 2, 3] >> sum()
let b = [1, 2, 3, 4] >> product()
  "
    a == 6
    b == 24
  );

  bang_test!(filter_and_map
  "
from list import { filter, map }
let [a, b] = [1, 2, 3] >> filter((x) => x >= 2)
let [c, d, e] = [1, 2, 3] >> map((x) => x * 2)
  "
    a == 2
    b == 3
    c == 2
    d == 4
    e == 6
  );

  bang_test!(max_min
  "
from list import { max, min }

let a = max([]) == -maths::INFINITY
let b = max([5])
let c = max([3, 2, 1])
let d = max([2, 1, 3])

let e = min([]) == maths::INFINITY
let f = min([1])
let g = min([3, 2, 1])
let h = min([2, 1, 3])
  "
  a == true
  b == 5
  c == 3
  d == 3
  e == true
  f == 1
  g == 1
  h == 1
  );
}

mod set {
  use super::*;

  bang_test!(size
"
from set import { set, size, isEmpty }

let a = size(set(1, 2, 3))
let b = size(set())
let c = size(3) == null
let d = isEmpty(set())
let e = isEmpty(set(1, 2, 3))
"
    a == 3
    b == 0
    c == true
    d == true
    e == false
  );

  bang_test!(insert_remove
"
from set import { set, insert, remove, size }

let a = set()
a >> insert(1)
let b = size(a)
a >> insert(88)
let c = size(a)
a >> insert(88)
let d = size(a)
a >> remove(4)
let e = size(a)
a >> remove(1)
let f = size(a)
"
    b == 1
    c == 2
    d == 2
    e == 2
    f == 1
  );

  bang_test!(includes
"
from set import { set, includes }

let a = set() >> includes(7)
let b = set(1, 2, 3) >> includes(2)
let c = set(1, 2, 3) >> includes(4)
"
    a == false
    b == true
    c == false
  );

  bang_test!(subset_superset_disjoint
"
from set import { set, isDisjoint, isSubset, isSuperset }

let a = set(1, 2, 3)
let b = set(1, 2)
let c = set(4, 5)

let x = isSubset(a, b)
let y = isSuperset(a, b)
let z = isDisjoint(a, c)
let w = isDisjoint(a, b)

"
    x == false
    y == true
    z == true
    w == false
  );

  bang_test!(union_difference_intersection_symmetric
"
from set import { set, size, union, difference, intersection, symmetricDifference }

let a = set(1, 2, 3)
let b = set(1, 2)
let c = set(4, 5)
let d = set(4, 6)

let ac = union(a, c)
let x = size(a)
let y = size(c)
let z = size(ac)

let e = difference(a, b) >> size()
let f = intersection(a, b) >> size()
let g = symmetricDifference(c, d) >> size()
"
    x == 3
    y == 2
    z == 5

    e == 1
    f == 2
    g == 2
  );

  bang_test!(to_list
"
from set import { set, toList }
from list import { length }

let a = set(1, 2, 3) >> toList()
let b = type(a)
let c = length(a)

let d = set(1,2,1) >> toList()
let e = length(d)
"
    b == "list"
    c == 3
    e == 2
  );

  bang_test!(cyclic_equals
  "
let a = set::new()
let b = set::new(a)
a >> set::insert(b)
let x = a == a
let y = a != b
      "
    x == true
    y == true
  );

  bang_test!(cyclic_to_string
    "
let a = set::new()
let b = set::new(a)

a >> set::insert(b)

// No assertion as set ordering isn't constant
let z = toString(a)
"
    z == "set(set(...))"
  );

  bang_test!(falsy
"
let a = 0
let b = 0

if (set::new())
  a += 1

let c = set::new()
c >> set::insert(5)
if (c)
  b += 1
"
    a == 0
    b == 1
  );
}

mod dict {
  use super::*;

  bang_test!(dict
"
from dict import { new, get, size, isEmpty, keys, values }
from list import { includes }

let a = new()
let b = size(a)
a['hello'] = 5
a['world'] = 3
let c = size(a)
let d = isEmpty(a)
let e = keys(a) >> includes('hello')
let f = keys(a) >> includes('world')
let g = values(a) >> includes(3)
let h = values(a) >> includes(5)
let i = a >> get('hello')
let j = (a >> get(44)) == null
"
    b == 0
    c == 2
    d == false
    e == true
    f == true
    g == true
    h == true
    i == 5
    j == true
  );

  bang_test!(dict_literal
  "
from dict import { new, get, size, isEmpty, keys, values }
from list import { includes }

let a = {}
let b = size(a)
a = {
  'hello': 5,
  'world': 3,
}
let c = size(a)
let d = isEmpty(a)
let e = keys(a) >> includes('hello')
let f = keys(a) >> includes('world')
let g = values(a) >> includes(3)
let h = values(a) >> includes(5)
let i = a >> get('hello')
let j = (a >> get(44)) == null
  "
      b == 0
      c == 2
      d == false
      e == true
      f == true
      g == true
      h == true
      i == 5
      j == true
    );

  bang_test!(falsy
"
let a = 0
let b = 0

if (dict::new())
  a += 1

let c = dict::new()
c['a'] = 1
if (c)
  b += 1
"
    a == 0
    b == 1
  );

  bang_test!(equality
"
let a = dict::new()
let b = dict::new()

let c = a == b

b[1] = 2
let d = a == b

a[1] = 2
let e = a ==b
"
    c == true
    d == false
    e == true
  );

  bang_test!(index_not_found
"
let a = dict::new()
let b = a['hello']
"
    RuntimeError
  );

  bang_test!(cyclic_values_equals
  "
let a = dict::new()
let b = dict::new()

b['hello'] = a
a['world'] = b

let x = a == a
let y = a != b
      "
    x == true
    y == true
  );

  bang_test!(cyclic_keys_equals
  "
let a = dict::new()
let b = dict::new()

a[a] = 7
b[a] = 7

let x = a == a
let y = a == b
        "
    x == true
    y == true
  );

  bang_test!(cyclic_keys_string
    "
let a = dict::new()
let b = dict::new()

a[a] = 7
b[a] = 4

let x = toString(a)
let y = toString(b)
        "
    x == "{ ...: 7 }"
    y == "{ { ...: 7 }: 4 }"
  );

  bang_test!(cyclic_keys_value
    "
let a = dict::new()
let b = dict::new()

a[7] = a
b[a] = 7

let x = toString(a)
let y = toString(b)
  "
    x == "{ 7: ... }"
    y == "{ { 7: ... }: 7 }"
  );
}
