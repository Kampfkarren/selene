local React

local function Component1()
    local a = {}
    React.useEffect(function()
        print(a)
        print(a.b)
        print(a[b])
    end, { a })
end

local function Component2()
    local a = {}
    React.useEffect(function()
        print(a) -- Bad
        print(a.b)
        print(a.b.c)
        print(a.b[c])
    end, { a.b })
end

local function Component()
    local a = {}
    React.useEffect(function()
        print(a) -- Bad
        print(a.b) -- Bad, but already reported with `a`
        print(a.b.c)
        print(a.b.c.d)
        print(a.b.c[d])
    end, { a.b.c })
end

local function Component()
    local a = {}
    React.useEffect(function()
        print(a.b) -- Bad
        print(a.b.c) -- Bad, but already reported with `a.b`
        print(a.b.c.d)
    end, { a.b.c.d })
end

local function Component()
    local a = {}
    local d = {}
    React.useEffect(function()
        print(a.b.c()) -- Good
        print(a.b.d()) -- Bad
        print(d.e.f.g()) -- Good
    end, { a.b.c, d.e.f })
end

local function Component()
    local a = {}
    React.useEffect(function()
        print(a.b["c d"]["e"])
    end, {})
end

local function Component()
    local a = {}
    React.useEffect(function()
        print(a["b c"]["d"])
    end, { a["b c"]["d"] })
end

local function Component()
    local a = {}
    React.useEffect(function()
        local _ = a["b c"]()
    end, {})
end
