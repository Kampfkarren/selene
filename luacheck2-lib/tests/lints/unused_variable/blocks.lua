local function unusedFunction()
    local unusedVariable = 1
end

local overidden = true

local function overridesIt()
    local overidden = false
    print(overidden)
end

overridesIt()
