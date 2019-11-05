# parenthese_conditions
## What it does
Checks for conditions in the form of `(expression)`.

## Why this is bad
Lua does not require these, and they are not idiomatic.

## Example
```lua
if (x) then
repeat until (x)
while (x) do
```

...should be written as...

```lua
if x then
repeat until x
while x do
```
