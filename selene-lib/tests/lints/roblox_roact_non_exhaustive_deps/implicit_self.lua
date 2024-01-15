local React

local function Component()
    local a = {}
    React.useEffect(function()
        local _ = a.b.c()
    end, { a.b.c })

    React.useEffect(function()
        -- Should require `a.b` as it is implicity passed
        local _ = a.b:c()
    end, { a.b.c })
end
