# shadowing
## What it does
Checks for overriding of variables under the same name.

## Why this is bad
This can cause confusion when reading the code when trying to understand which variable is being used, and if you want to use the original variable you either have to redefine it under a temporary name or refactor the code that shadowed it.

## Configuration
`ignore_pattern` (default: `"^_"`) - A [regular expression](https://en.wikipedia.org/wiki/Regular_expression) that is used to specify names that are allowed to be shadowed. The default allows for variables like `_` to be shadowed, as they shouldn't be used anyway.

## Example
```lua
local x = 1

if foo then
    local x = 1
end
```
