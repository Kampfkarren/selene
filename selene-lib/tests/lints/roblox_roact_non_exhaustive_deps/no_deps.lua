local React

local function Component()
    local a = {}

    React.useEffect(function()
        print(a)
    end)
end
