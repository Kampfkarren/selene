-- Test cases for useMemo

local React = {
    useMemo = function() end,
}

-- Missing dependency in useMemo
local function Component1(props)
    local computed = React.useMemo(function()
        return props.value * 2
    end, {})
end

-- Correct dependencies in useMemo
local function Component2(props)
    local multiplier = 5
    
    local computed = React.useMemo(function()
        return props.value * multiplier
    end, { props.value, multiplier })
end

-- Unnecessary dependency in useMemo
local function Component3()
    local unused = 5
    
    local computed = React.useMemo(function()
        return 10 * 2
    end, { unused })
end

-- Complex calculation with missing deps
local function Component4(props)
    local base = 10
    
    local computed = React.useMemo(function()
        local result = props.value + base
        return result * props.multiplier
    end, { base })
end

