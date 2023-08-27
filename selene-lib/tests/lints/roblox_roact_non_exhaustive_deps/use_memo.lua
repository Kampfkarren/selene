local React

local function Component(props)
    React.useEffect(function()
        print(props)
    end, {})
end
