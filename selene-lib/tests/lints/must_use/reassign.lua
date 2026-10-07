local math2 = math
local custom_math = {}

function custom_math.max(a, b)
    return a + b
end

-- This should not fail: once math2 is reassigned, math.max's must_use no longer applies.
math2 = custom_math
math2.max(1, 2)
