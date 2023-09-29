# manual_table_clone
## What it does
Detects manual re-implementations of `table.clone` when it exists in the standard library.

## Why this is bad
`table.clone` is much simpler to read and faster than manual re-implementations.

## Example
```lua
local output = {}

for key, value in pairs(input) do
    output[key] = value
end
```

...should be written as...

```lua
local output = table.clone(input)
```

## Remarks
Very little outside this exact pattern is matched. This is the list of circumstances which will stop the lint from triggering:

- Any logic in the body of the function aside from `output[key] = value`.
- Any usage of the output variable in between the definition and the loop (as determined by position in code).
- If the input variable is not a plain locally initialized variable. For example, `self.state[key] = value` will not lint.
- If the input variable is not defined as a completely empty table.
- If the loop and input variable are defined at different depths.

---

The detected looping patterns are `pairs(t)`, `ipairs(t)`, `next, t`, and `t` (Luau generalized iteration). If `ipairs` is used, `table.clone` is not an exact match if the table is not exclusively an array. For example:

```lua
local mixedTable = { 1, 2, 3 }
mixedTable.key = "value"

local clone = {}

-- Lints, but is not equivalent, since ipairs only loops over the array part.
for key, value in ipairs(mixedTable) do
    clone[key] = value
end
```

When `ipairs` is the function being used, you'll be notified of this potential gotcha.
