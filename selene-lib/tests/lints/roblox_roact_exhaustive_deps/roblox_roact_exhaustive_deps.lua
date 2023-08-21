local React

local function Component1()
    useEffect(function()
        local allowed = a.b.c

        local disallowed = d + e
        d = 3
        local nowallowed = d
    end, { a.b, c })
end

local function Component2()
    useEffect(function()
        notallowed(a, `{b}`)
    end, {})
end

local function Component3()
    useEffect(function()
        a(b(c(d)))
    end, {})
end
