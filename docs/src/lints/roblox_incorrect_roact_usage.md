# roblox_incorrect_roact_usage
## What it does
Checks for valid uses of createElement. Verifies that class name given is valid and that the properties passed for it are valid for that class.

## Why this is bad
This is guaranteed to fail once it is rendered. Furthermore, the createElement itself will not error--only once it's mounted will it error.

## Example
```lua
-- Using react-lua
React.createElement("Frame", {
    key = "Valid property for React",
})

-- Using legacy Roact
Roact.createElement("Frame", {
    key = "Invalid property for Roact",
    ThisPropertyDoesntExist = true,
    Name = "This property should not be passed in",

    [Roact.Event.ThisEventDoesntExist] = function() end,
})

Roact.createElement("BadClass", {})
```

## Remarks
This lint is naive and makes several assumptions about the way you write your code. The assumptions are based on idiomatic Roact.

1. It assumes you are either calling `createElement` directly or creating a local variable that's assigned to `[Roact/React].createElement`.
2. It assumes if you are using a local variable, you're not reassigning it.
3. It assumes either Roact or React is defined. [`undefined_variable`](./undefined_variable.md) will still lint, however.

This lint assumes legacy Roact if the library name is Roact and react-lua if the library name is React.

This lint does not verify if the value you are giving is correct, so `Text = UDim2.new()` will be treated as correct. This lint, right now, only checks property and class names.

This lint is only active if you are using the Roblox standard library.
