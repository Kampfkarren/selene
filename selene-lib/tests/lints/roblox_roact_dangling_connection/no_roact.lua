a:Connect()
a.Connect()
a:connect()
a.connect()

a.b:Connect()
a.b.Connect()
a.b:connect()
a.b.connect()

a.b.c:Connect()
a.b.c.Connect()
a.b.c:connect()
a.b.c.connect()

local foo = a:Connect()
local foo = a.Connect()
local foo = a:connect()
local foo = a.connect()

foo = a:Connect()
foo = a.Connect()
foo = a:connect()
foo = a.connect()

foo = a.b:Connect()
foo = a.b.Connect()
foo = a.b:connect()
foo = a.b.connect()

foo = a.b.c:Connect()
foo = a.b.c.Connect()
foo = a.b.c:connect()
foo = a.b.c.connect()

local function c()
    a:Connect()
    a:connect()

    a.b:Connect()
    a.b:connect()

    a.b.c:Connect()
    a.b.c:connect()

    foo = a:Connect()
    foo = a:connect()

    foo = a.b:Connect()
    foo = a.b:connect()

    foo = a.b.c:Connect()
    foo = a.b.c:connect()
end

function d:e()
    a:Connect()
    a:connect()

    a.b:Connect()
    a.b:connect()

    a.b.c:Connect()
    a.b.c:connect()

    foo = a:Connect()
    foo = a:connect()

    foo = a.b:Connect()
    foo = a.b:connect()

    foo = a.b.c:Connect()
    foo = a.b.c:connect()
end
