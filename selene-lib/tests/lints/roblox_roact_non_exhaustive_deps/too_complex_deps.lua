local React

local function Component()
    local a = {}
    React.useEffect(function()
        print(a[b])
    end, { a[b] })
end

local function Component()
    local a = {}
    React.useEffect(function()
        print(a.b["c"]())
    end, { a.b["c"]() })
end

local function Component()
    local a = {}
    React.useEffect(function()
        print(a())
        -- FIXME: false negatives for function calls without indexing
    end, { a() })
end

local function Component()
    local a = {}
    React.useEffect(function()
        -- A function call in place of array brackets should NOT warn about complex expr
        -- due to lua-specific pattern of helper functions to patch holes in arrays
    end, a.b())

    React.useEffect(function()
        -- Now this should warn
    end, a.b(a.b()))

    React.useEffect(function()
        -- Now this should warn
    end, { a.b() })
end
