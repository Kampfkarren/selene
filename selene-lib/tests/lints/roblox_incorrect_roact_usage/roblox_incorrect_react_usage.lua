React.createElement("Frame", {
    ThisPropertyDoesntExist = true,
    Size = UDim2.new(1, 0, 1, 0),
    ref = true,
    children = true,
    key = true,

    [React.Event.InputBegan] = function()
    end,

    [React.Event.ThisEventDoesntExist] = function()
    end,
})

local e = React.createElement

e("Frame", {
    Size = UDim2.new(1, 0, 1, 0),
    ThisPropertyDoesntExist = true,
})

e("ThisDoesntExist", {})

e(Components.FooComponent, {
    Foo = 1,
})

call("foo", {})

e("ThisDoesntExist")

e(Components.FooComponent, {
    Name = "Can be passed",
})

React.createElement(Components.FooComponent, {
    Name = "Can be passed",
})

e("Frame", {
    Name = "Should not be passed",
})

e("Frame", {
    Name = "Should.not.be.passed",
})

e("Frame", {
    Name = "0Should0not0be0passed",
})

e("Frame", {
    Name = "Should0not0be0passed",
})

e("Frame", {
    Name = "_Should_not_be_passed",
})

e("Frame", {
    Name = "ShouldNotBePassed",
})

React.createElement("Frame", {
    Name = a,
})
