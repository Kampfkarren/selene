# unused_variable
## What it does
Checks for variables that are unused.

## Why this is bad
The existence of unused variables could indicate buggy code.

## Configuration
`allow_unused_self` (default: `true`) - A bool that determines whether not using `self` in a method function (`function Player:SwapWeapons()`) is allowed.

`ignore_pattern` (default: `"^_"`) - A [regular expression](https://en.wikipedia.org/wiki/Regular_expression) for variables that are allowed to be unused. The default allows for variables like `_` to be unused, as they shouldn't be used anyway.

## Example
```lua
local foo = 1
```

## Remarks

### `_` prefixing
If you intend to create a variable without using it, replace it with `_` or something that starts with `_`. You'll see this most in generic for loops.

```lua
for _, value in ipairs(list) do
```

### `observes`
Standard libraries can apply [an `observes` field](../usage/std.md#observes) to distinguish an argument from being only written to.

This is so that we can get lints like the following:

```lua
local writtenOnly = {}
table.insert(writtenOnly, 1)
```

```
warning[unused_variable]: writtenOnly is assigned a value, but never used
  ┌─ example.lua:1:7
  │
1 │ local writtenOnly = {}
  │       ^^^^^^^^^^^
2 │ table.insert(writtenOnly, 1)
  │              ----------- `table.insert` only writes to `writtenOnly`
```

This only applies when the function call is its own statement. So for instance, this:

```lua
local list = {}
print(table.insert(list, 1))
```

...will *not* lint. To understand this, consider if `table.insert` returned the index. Without this check, this code:

```lua
local list = {}
local index = table.insert(list, 1)
```

...would lint `list` as mutated only, which while technically true, is unimportant considering `index` is affected by the mutation.

This also requires that the variable be a static table. This:

```lua
return function(value)
    table.insert(value, 1)
end
```

...will not lint, as we cannot be sure `value` is unused outside of this.
