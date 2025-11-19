-- Test cases for missing dependencies

local React = {
    useEffect = function() end,
}

-- Missing single dependency
local function Component1(props)
    React.useEffect(function()
        print(props.value)
    end, {})
end

-- Missing multiple dependencies
local function Component2(props)
    local count = 5
    
    React.useEffect(function()
        print(props.value)
        print(count)
        print(props.name)
    end, {})
end

-- Missing dependency with some correct ones
local function Component3(props)
    local count = 5
    local name = "test"
    
    React.useEffect(function()
        print(props.value)
        print(count)
        print(name)
    end, { count })
end

-- Missing dependency in nested function call
local function Component4(props)
    local spawn = function() end
    React.useEffect(function()
        spawn(function()
            print(props.value)
        end)
    end, {})
end

-- Missing dependency with property access
local function Component5(props)
    React.useEffect(function()
        print(props.data.value)
    end, {})
end

-- Roact version
local Roact = {
    useEffect = function() end,
}

local function RoactComponent(props)
    Roact.useEffect(function()
        print(props.value)
    end, {})
end

