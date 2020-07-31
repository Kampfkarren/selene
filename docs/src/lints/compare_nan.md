# compare_nan
## What it does
Checks for comparison to `0/0`.

## Why this is bad
The most common case of comparing against [nan](https://en.wikipedia.org/wiki/NaN) is to check if a variable is nan. In this case, you do not want to compare to `0/0` directly and instead want to compare the variable to itself.

## Example
```lua
print(x == 0/0)
print(x ~= 0/0)
```

...should be written as...
```lua
print(x ~= x)
print(x == x)
```
