local React
local useEffect = React.useEffect

a:connect()

local function c()
    a:Connect()
    a:connect()
    a:ConnectParallel()
    a:Once()

    a.Connect()
    a.connect()
    a.ConnectParallel()
    a.Once()

    -- Ignore since `a` might take ownership of connections
    a(b:Connect())
    a(function() end, b:Connect())

    useEffect(function()
        a:connect()
        a(b:Connect())
        local b = a:connect()
    end)

    React.useEffect(function()
        a:connect()
        a(b:Connect())
        local b = a:connect()
    end)
end
