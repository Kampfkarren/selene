# roblox_incorrect_react_usage

## Valid class name and props

### What it does
Verifies that class name given is valid and that the properties passed for it are valid for that class.

### Why this is bad
This is guaranteed to fail once it is rendered. Furthermore, the createElement itself will not error--only once it's mounted will it error.

### Example
```lua
React.createElement("Frame", {
    key = "This exists",
    ThisPropertyDoesntExist = true,

    [React.Event.ThisEventDoesntExist] = function() end,
})

React.createElement("BadClass", {})
```

## No name prop

### What it does
Verifies that `Name` is not passed in as props to Roblox elements.

### Why this is bad
Elements are named by their keys.

### Example
```lua
React.createElement("Frame", {
    Name = "This property should not be passed in",
})
```

## No children prop

### What it does
Verifies that `children` is not passed in as props to components.

### Why this is bad
The children should be passed as additional arguments to `React.createElement`.

```lua
React.createElement("Frame", {
    children = props.children,
})
```

## Remarks
This lint is naive and makes several assumptions about the way you write your code. The assumptions are based on idiomatic React.

1. It assumes you are either calling `React.createElement` directly or creating a local variable that's assigned to `React.createElement`.
2. It assumes if you are using a local variable, you're not reassigning it.
3. It assumes React is defined. [`undefined_variable`](./undefined_variable.md) will still lint, however.

This lint does not verify if the value you are giving is correct, so `Text = UDim2.new()` will be treated as correct. This lint, right now, only checks property and class names.

This lint is only active if you are using the Roblox standard library.
