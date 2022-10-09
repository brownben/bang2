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
"
  a == "number"
  b == "boolean"
  c == "null"
  d == "function"
  e == "string"
  f == "list"
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
  CompileError
);

bang_test!(unknown_module_value
  "from maths import { stuff }"
  CompileError
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
}
