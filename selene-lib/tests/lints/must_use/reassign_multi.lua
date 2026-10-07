local a = math
local b = {}

-- This should fail: after swapping, b should now refer to math.
a, b = b, a
b.max(x, y)

local c = math
local d = {}

-- This should fail too: the second RHS `c` must read the outer `c`, not the new local `c`.
local c, d = d, c
d.max(x, y)
