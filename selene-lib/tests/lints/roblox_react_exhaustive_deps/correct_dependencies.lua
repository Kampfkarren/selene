-- Test cases for correct dependencies (should have no warnings)

local React = {
    useEffect = function() end,
}

-- Correct single dependency
local function Component1(props)
    React.useEffect(function()
        print(props.value)
    end, { props.value })
end

-- Correct multiple dependencies
local function Component2(props)
    local count = 5
    
    React.useEffect(function()
        print(props.value)
        print(count)
    end, { props.value, count })
end

-- No dependencies needed
local function Component3()
    React.useEffect(function()
        print("hello")
    end, {})
end

-- No dependencies array (runs every render)
local function Component4(props)
    React.useEffect(function()
        print(props.value)
    end)
end

-- Using only constants
local function Component5()
    React.useEffect(function()
        print(1 + 2)
        print("constant")
    end, {})
end

-- Using built-in globals
local function Component6()
    React.useEffect(function()
        print("test")
        local t = { 1, 2, 3 }
        table.insert(t, 4)
    end, {})
end

