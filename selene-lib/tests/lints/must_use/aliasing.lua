local math2 = math

-- This should fail: math2 aliases math, and math.max is marked must_use.
math2.max(x, y)

-- This should fail too: plain assignment should preserve the alias even without `local`.
math_global = math
math_global.max(x, y)
