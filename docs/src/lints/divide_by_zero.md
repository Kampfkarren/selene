# divide_by_zero
## What it does
Checks for division by zero. Allows `0 / 0` as a way to get [nan](https://en.wikipedia.org/wiki/NaN).

## Why this is bad
`n / 0` equals `math.huge` when n is positive, and `-math.huge` when n is negative. Use these values directly instead, as using the `/ 0` way is confusing to read and non-idiomatic.

## Example
```lua
print(1 / 0)
print(-1 / 0)
```

...should be written as...
```lua
print(math.huge)
print(-math.huge)
```
