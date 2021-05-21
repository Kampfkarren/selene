# duplicate_keys
## What it does
Checks for duplicate keys being defined inside of tables.

## Why this is bad
Tables with a key defined more than once will only use one of the values.

## Example
```lua
local foo = {
    a = 1,
    b = 5,
    ["a"] = 3, -- duplicate definition
    c = 3,
    b = 1, -- duplicate definition
}

local bar = {
    "foo",
    "bar",
    [1524] = "hello",
    "baz",
    "foobar",
    [2] = "goodbye", -- duplicate to `bar` which has key `2`
}
```

## Remarks
Only handles keys which constant string/number literals or named (such as `{ a = true }`).
Array-like values are also handled, where `{"foo"}` is implicitly handled as `{ [1] = "foo" }`.
