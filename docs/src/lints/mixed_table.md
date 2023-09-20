# mixed_table
## What it does
Checks for mixed tables (tables that act as both an array and dictionary).

## Why this is bad
Mixed tables harms readability and are prone to bugs. There is almost always a better alternative.

## Example
```lua
local foo = {
    "array field",
    bar = "dictionary field",
}
```
