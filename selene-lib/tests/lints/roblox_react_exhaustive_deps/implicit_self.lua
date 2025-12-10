-- Test cases for implicit self in method calls

local React = {
    useEffect = function() end,
}

-- Method call with colon passes self implicitly
local function Component1()
    local a = {}
    
    React.useEffect(function()
        -- a.b:c() implicitly passes a.b as self
        -- So a.b should be in deps, not a.b.c
        local _ = a.b:c()
    end, { a.b.c })
end

-- Regular function call (dot) doesn't pass self
local function Component2()
    local a = {}
    
    React.useEffect(function()
        -- a.b.c() is just a function call, a.b.c is correct dep
        local _ = a.b.c()
    end, { a.b.c })
end

-- Method call with correct deps
local function Component3()
    local a = {}
    
    React.useEffect(function()
        local _ = a.b:c()
    end, { a.b })
end

