# global_usage
## What it does
Prohibits use of `_G`.

## Why this is bad
`_G` is global mutable state, which is heavily regarded as harmful. You should instead refactor your code to be more modular in nature.

## Remarks
If you are using the Roblox standard library, use of `shared` is prohibited under this rule.

## Example
```lua
_G.foo = 1
```
