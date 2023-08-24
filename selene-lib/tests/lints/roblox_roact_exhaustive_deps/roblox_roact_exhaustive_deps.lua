local React

local notreactive1 = {}

local function Component1()
    local reactive1 = {}
    local reactive2 = {}

    React.useEffect(function()
        local allowed = notreactive1
        local notallowed = reactive1
        print(reactive2)
    end, {})
end

local function Component2()
    local reactive1 = {}

    React.useEffect(function()
        allowed(reactive1, `{notreactive1}`)
    end, { reactive1 })
end

local function Component3()
    local reactive1 = {}
    local reactive2 = {}

    React.useEffect(function()
        notreactive1(reactive1(reactive2()))
    end, depArray(reactive1))
end

local function Component4()
    local _, setState = React.useState()

    React.useEffect(function()
        setState()
    end, {})
end

local function Component5()
    local reactive1 = {}

    React.useEffect(function()
        local notreactive = function()
            if true then
                local notallowed = reactive1
            end
        end
        notreactive()
    end, {})
end

local function MakeComponent()
    local notreactive = {}

    local function Component()
        local reactive1 = {}

        React.useEffect(function()
            local allowed = notreactive
            local allowed = reactive1
        end, { reactive1 })
    end
end
