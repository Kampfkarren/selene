# must_use
## What it does
Checks that the return values of functions [marked `must_use`](../usage/std.md#must_use) are used.

## Why this is bad
This lint will only catch uses where the function has no reason to be called other than to use its output.


## Example
```lua
bit32.bor(entity.flags, Flags.Invincible)
```

...should be written as...

```lua
entity.flags = bit32.bor(entity.flags, Flags.Invincible)
```

...as `bit32.bor` only produces a new value, it does not mutate anything.

## Remarks
The output is deemed "unused" if the function call is its own statement.
