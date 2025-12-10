-- Test cases for known stable variables (setState, refs, etc.)

local React = {
    useEffect = function() end,
    useState = function() end,
    useRef = function() end,
    useBinding = function() end,
}

-- setState from useState should be stable and not required in deps
local function Component1()
    local state, setState = React.useState()
    
    React.useEffect(function()
        -- setState is stable, shouldn't need to be in deps
        setState("new value")
    end, {})
end

-- state from useState SHOULD be in deps
local function Component2()
    local state, setState = React.useState()
    
    React.useEffect(function()
        -- state is NOT stable, should be in deps
        print(state)
    end, {})
end

-- ref from useRef should be stable
local function Component3()
    local ref = React.useRef()
    
    React.useEffect(function()
        -- ref is stable, shouldn't need to be in deps
        print(ref.current)
    end, {})
end

-- setBinding from useBinding should be stable
local function Component4()
    local binding, setBinding = React.useBinding()
    
    React.useEffect(function()
        -- setBinding is stable
        setBinding("new value")
    end, {})
end

-- Passing stable vars in deps shouldn't warn about them being unnecessary
local function Component5()
    local state, setState = React.useState()
    
    React.useEffect(function()
        setState("value")
    end, { setState })
end

