# suspicious_reverse_loop
## What it does
Checks for `for _ = #x, 1 do` sequences without specifying a negative step.

## Why this is bad
This loop will only run at most once, instead of going in reverse. If you truly did mean to run your loop only once, just use `if #x > 0` instead.

## Example
```lua
for _ = #x, 1 do
```

...should be written as...

```lua
for _ = #x, 1, -1 do
```
