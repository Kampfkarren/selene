# if_same_then_else
## What it does
Checks for branches in if blocks that are equivalent.

## Why this is bad
This is most likely a copy and paste error.

## Example
```lua
if foo then
    print(1)
else
    print(1)
end
```
