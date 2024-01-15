local React
local useEffect = React.useEffect

a:connect()

local function Component()
    a:Connect()

    useEffect(function()
        -- Ignore since `a` might take ownership of connections
        a(b:Connect())
        a(function() end, b:Connect())

        a:Connect()
        a:connect()

        a.b:Connect()
        a.b:connect()

        a.b.c:Connect()
        a.b.c:connect()

        good = a:Connect()
        good = a:connect()

        good = a.b:Connect()
        good = a.b:connect()

        good = a.b.c:Connect()
        good = a.b.c:connect()
    end)

    React.useEffect(function()
        a(b:Connect())
        local b = a:connect()

        a:connect()
    end)
end
