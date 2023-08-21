local React

local function Component1()
    React.useEffect(function()
        local allowed = a.b.c

        local disallowed = d + e
        d = 3
        local nowallowed = d
    end, { a.b, c })
end

local function Component2()
    React.useEffect(function()
        notallowed(a, `{b}`)
    end, {})
end

local function Component3()
    React.useEffect(function()
        a(b(c(d)))
    end, {})
end

local function Component4()
    local _, setState = React.useState()

    React.useEffect(function()
        setState()
    end, {})
end
