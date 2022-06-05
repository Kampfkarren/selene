# deprecated
## What it does
Checks for use of deprecated fields and functions, as configured [by your standard library](../usage/std.md#deprecated).

## Why this is bad
Deprecated fields may not be getting any support, or even face the possibility of being removed.

## Example
```lua
local count = table.getn(x)
```

...should be written as...

```lua
local count = #x
```
