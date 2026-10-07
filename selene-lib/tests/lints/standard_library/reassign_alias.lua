local ecs = {}

-- This should fail: reassignment to z2.ecs should re-establish the std alias.
ecs = z2.ecs
local id = ecs.create_entity(3)
