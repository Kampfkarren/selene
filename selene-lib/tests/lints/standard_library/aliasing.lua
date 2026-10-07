local ecs = z2.ecs
local ecs2 = ecs

-- This should not fail: ecs aliases z2.ecs, and create_entity is called correctly.
local id = ecs.create_entity()
-- This should fail: ecs2 aliases z2.ecs, so create_entity still expects 0 args.
local e = ecs2.create_entity(3)
-- This should fail: ecs2 still points at z2.ecs, so misspelled fields should be checked too.
local g = ecs2.create_enztity()

-- This should fail too: plain assignment should preserve the alias even without `local`.
ecs_global = z2.ecs
local h = ecs_global.create_entity(3)

local z2 = {}
local ecs3 = z2
-- This should not fail: this z2 is just a local table, so create_entity is not checked as a std function.
local f = ecs3.create_entity(3)
