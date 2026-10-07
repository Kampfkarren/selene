local ecs = z2.ecs

-- This should not fail: once ecs is reassigned, it should no longer be checked as z2.ecs.
ecs = {}
local id = ecs.create_entity(3)
