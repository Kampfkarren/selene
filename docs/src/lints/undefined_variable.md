# undefined_variable
## What it does
Checks for uses of variables that are not defined.

## Why this is bad
This is most likely a typo.

## Example
```lua
-- vv oops!
prinnt("hello, world!")
```

## Remarks
If you are using a different standard library where a global variable is defined that selene isn't picking up on, create a [standard library](../usage/std.md) that specifies it.
