# global_usage
## What it does
Prohibits use of `_G`.

## Why this is bad
`_G` is global mutable state, which is heavily regarded as harmful. You should instead refactor your code to be more modular in nature.

## Configuration
`ignore_pattern` - A [regular expression](https://en.wikipedia.org/wiki/Regular_expression) for variables that are allowed to be global variables. The default disallows all global variables regardless of their name.

## Remarks
If you are using the Roblox standard library, use of `shared` is prohibited under this lint.

## Example
```lua
_G.foo = 1
```
