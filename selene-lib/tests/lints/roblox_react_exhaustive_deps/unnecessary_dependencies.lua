-- Test cases for unnecessary dependencies

local React = {
    useEffect = function() end,
}

-- Single unnecessary dependency
local function Component1(props)
    React.useEffect(function()
        print("hello")
    end, { props.value })
end

-- Multiple unnecessary dependencies
local function Component2(props)
    React.useEffect(function()
        print("hello")
    end, { props.value, props.name, props.id })
end

-- Mix of correct and unnecessary dependencies
local function Component3(props)
    local count = 5
    
    React.useEffect(function()
        print(count)
    end, { count, props.value })
end

-- All dependencies unnecessary
local function Component4()
    local a = 1
    local b = 2
    
    React.useEffect(function()
        print("constant")
    end, { a, b })
end

