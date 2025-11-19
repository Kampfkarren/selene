-- Test cases for property access dependencies

local React = {
    useEffect = function() end,
}

-- Property access - correct
local function Component1(props)
    React.useEffect(function()
        print(props.data.value)
    end, { props.data.value })
end

-- Property access - missing
local function Component2(props)
    React.useEffect(function()
        print(props.user.name)
        print(props.user.age)
    end, {})
end

-- Nested property access
local function Component3(props)
    React.useEffect(function()
        print(props.data.user.profile.name)
    end, { props.data.user.profile.name })
end

-- Mixed property and direct access
local function Component4(props)
    local count = 5
    
    React.useEffect(function()
        print(props.data.value)
        print(count)
    end, { count })
end

