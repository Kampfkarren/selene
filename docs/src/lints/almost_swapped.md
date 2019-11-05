# almost_swapped
## What it does
Checks for `foo = bar; bar = foo` sequences.

## Why this is bad
This looks like a failed attempt to swap.

## Example
```lua
a = b
b = a
```

...should be written as...

```lua
a, b = b, a
```
