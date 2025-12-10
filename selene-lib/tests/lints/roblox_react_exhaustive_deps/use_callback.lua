-- Test cases for useCallback

local React = {
    useCallback = function() end,
}

-- Missing dependency in useCallback
local function Component1(props)
    local handleClick = React.useCallback(function()
        print(props.value)
    end, {})
end

-- Correct dependencies in useCallback
local function Component2(props)
    local count = 5
    
    local handleClick = React.useCallback(function()
        print(props.value)
        print(count)
    end, { props.value, count })
end

-- Unnecessary dependency in useCallback
local function Component3()
    local value = 5
    
    local handleClick = React.useCallback(function()
        print("constant")
    end, { value })
end

-- Mixed issues
local function Component4(props)
    local count = 5
    local unused = 10
    
    local handleClick = React.useCallback(function()
        print(props.value)
        print(count)
    end, { unused })
end

