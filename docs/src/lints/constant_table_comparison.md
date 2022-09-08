# constant_table_comparison
## What it does
Checks for direct comparisons with constant tables.

## Why this is bad
This will always fail.

## Example
```lua
if x == { "a", "b", "c" } then
```

...will never pass.

```lua
if x == {} then
```

...should be written as...
```lua
if next(x) == nil then
```
