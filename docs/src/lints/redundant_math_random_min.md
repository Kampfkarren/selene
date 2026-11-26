# redundant_math_random_min
## What it does
Checks for calls to `math.random` where the lower bound is `1`.

## Why this is bad
`math.random(n)` already returns an integer from 1 to `n`, so passing `1` as the lower bound is redundant.

## Example
```lua
local index = math.random(1, #items)
```

...should be written as...

```lua
local index = math.random(#items)
```
