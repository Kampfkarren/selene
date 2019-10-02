-- Functions
local function unusedFunction()
    local unusedVariableA = 1
    local unusedVariableB = 1
end

print(unusedVariableB)
local overidden = true

local function overridesIt()
    local overidden = false
    print(overidden)
end

overridesIt()
