# unused_variable
## What it does
Checks for variables that are unused.

## Why this is bad
The existence of unused variables could indicate buggy code.

## Configuration
`allow_unused_self` (default: `false`) - A bool that determines whether not using `self` in a method function (`function Player:SwapWeapons()`) is allowed.

`ignore_pattern` (default: `"^_"`) - A [regular expression](https://en.wikipedia.org/wiki/Regular_expression) for variables that are allowed to be unused. The default allows for variables like `_` to be unused, as they shouldn't be used anyway.

## Example
```lua
local foo = 1
```

## Remarks
If you intend to create a variable without using it, replace it with `_` or something that starts with `_`. You'll see this most in generic for loops.

```lua
for _, value in ipairs(list) do
```
