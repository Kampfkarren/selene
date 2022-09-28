# deprecated
## What it does
Checks for use of deprecated fields and functions, as configured [by your standard library](../usage/std.md#deprecated).

## Why this is bad
Deprecated fields may not be getting any support, or even face the possibility of being removed.

## Configuration
`allow` - A list of patterns where the deprecated lint will not throw. For instance, `["table.getn"]` will allow you to use `table.getn`, even though it is deprecated. This supports wildcards, so `table.*` will allow both `table.getn` and `table.foreach`.

## Example
```lua
local count = table.getn(x)
```

...should be written as...

```lua
local count = #x
```
