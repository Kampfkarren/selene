# unbalanced_assignments
## What it does
Checks for unbalanced assignments, such as `a, b, c = 1`.

## Why this is bad
You shouldn't declare variables you're not immediately initializing on the same line as ones you are. This is most likely just forgetting to specify the rest of the variables.

## Example
```lua
a, b, c = 1
a = 1, 2
```

## Remarks
There are a few things this lint won't catch.

`a, b, c = call()` will not lint, as `call()` could return multiple values.

`a, b, c = call(), 2` will, however, as you will only be using the first value of `call()`. You will even receive a helpful message about this.

```
error[unbalanced_assignments]: values on right side don't match up to the left side of the assignment

   ┌── unbalanced_assignments.lua:6:11 ───
   │
 6 │ a, b, c = call(), 2
   │           ^^^^^^^^^
   │

   ┌── unbalanced_assignments.lua:6:11 ───
   │
 6 │ a, b, c = call(), 2
   │           ------ help: if this function returns more than one value, the only first return value is actually used
   │
```

If nil is specified as the last value, the rest will be ignored. This means...

```lua
a, b, c = nil
```

...will not lint.
