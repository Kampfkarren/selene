# ifs_same_cond
## What it does
Checks for branches in if blocks with equivalent conditions.

## Why this is bad
This is most likely a copy and paste error.

## Example
```lua
if foo then
    print(1)
elseif foo then
    print(1)
end
```

## Remarks
This ignores conditions that could have side effects, such as function calls. This will not lint:

```lua
if foo() then
    print(1)
elseif foo() then
    print(1)
end
```

...as the result of `foo()` could be different the second time it is called.
