local a = z2.ecs
local b = {}

-- This should fail: after swapping, b should now refer to z2.ecs.
a, b = b, a
local id = b.create_entity(3)

local c = z2.ecs
local d = {}

-- This should fail too: the second RHS `c` must read the outer `c`, not the new local `c`.
local c, d = d, c
local id2 = d.create_entity(3)
