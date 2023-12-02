# approx_constant
## What it does
Checks for number literals that approximate constants.

## Why this is bad
Using constants provided by the Lua standard library is more precise.

## Example
```lua
local x = 3.14
```

...should be written as...

```lua
local x = math.pi
```
