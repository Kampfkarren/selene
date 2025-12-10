-- Test cases for outer scope variables

local React = {
    useEffect = function() end,
}

-- Module-level variable (outer scope)
local moduleVar = {}

local function Component1()
    local reactiveVar = {}
    
    React.useEffect(function()
        -- moduleVar is outer scope, doesn't need to be in deps
        print(moduleVar)
        -- reactiveVar is component scope, needs to be in deps
        print(reactiveVar)
    end, {})
end

-- Outer scope variable in deps should warn as unnecessary
local function Component2()
    React.useEffect(function()
        print(moduleVar)
    end, { moduleVar })
end

-- Nested component - outer component's vars are outer scope
local function MakeComponent()
    local outerVar = {}
    
    local function Component()
        local innerVar = {}
        
        React.useEffect(function()
            -- outerVar is from outer scope (not component scope)
            print(outerVar)
            -- innerVar is component scope
            print(innerVar)
        end, { innerVar })
    end
end

