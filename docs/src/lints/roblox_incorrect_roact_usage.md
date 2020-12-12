# roblox_incorrect_roact_usage
## What it does
Checks for valid uses of Roact.createElement. Verifies that class name given is valid and that the properties passed for it are valid for that class.

## Why this is bad
This is guaranteed to fail once it is rendered. Furthermore, the createElement itself will not error--only once its mounted will it error.

## Example
```lua
Roact.createElement("Frame", {
    ThisPropertyDoesntExist = true,
})

Roact.createElement("BadClass", {})
```

## Remarks
This lint is naive and makes several assumptions about the way you write your code. The assumptions are based on idiomatic Roact.

1. It assumes you are either calling `Roact.createElement` directly or creating a local variable that's assigned to `Roact.createElement`.
2. It assumes if you are using a local variable, you're not reassigning it.
3. It assumes Roact is defined. [`undefined_variable`](./undefined_variable.md) will still lint, however.

This lint does not verify if the value you are giving is correct, so `Text = UDim2.new()` will be treated as correct. This lint, right now, only checks property and class names.

Additionally, this lint is based off of [rbx_reflection](https://docs.rs/rbx_reflection/3/rbx_reflection/). In practice, this means that if Roblox adds new properties or classes, selene will not know they exist until you update it.

This lint is only active if you are using the Roblox standard library.
