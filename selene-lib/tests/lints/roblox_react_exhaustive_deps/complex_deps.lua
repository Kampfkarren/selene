-- Test cases for complex dependency patterns

local React = {
    useEffect = function() end,
}

-- Bracket indexing with string literals
local function Component1()
    local a = {}
    
    React.useEffect(function()
        print(a["b c"]["d"])
    end, { a["b c"]["d"] })
end

-- Missing bracket-indexed dependency
local function Component2()
    local a = {}
    
    React.useEffect(function()
        print(a["key"])
    end, {})
end

-- Hierarchical: using a.b.c but only a.b in deps
local function Component3()
    local a = {}
    
    React.useEffect(function()
        print(a)
        print(a.b)
        print(a.b.c)
    end, { a.b })
end

-- Hierarchical: using a but only a.b in deps (should warn about a)
local function Component4()
    local a = {}
    
    React.useEffect(function()
        print(a)
        print(a.b)
    end, { a.b })
end

-- Multiple property paths
local function Component5()
    local a = {}
    local d = {}
    
    React.useEffect(function()
        print(a.b.c())
        print(a.b.d())
        print(d.e.f.g())
    end, { a.b.c, d.e.f })
end

