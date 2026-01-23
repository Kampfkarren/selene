local types = require(script.types)

local function f<T>() end
local value = f<<types.something>>()
