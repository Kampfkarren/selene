local math2 = {}

-- This should fail: reassignment to math should re-establish math.max's must_use behavior.
math2 = math
math2.max(x, y)
