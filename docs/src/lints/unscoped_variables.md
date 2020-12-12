# unscoped_variables
## What it does
Checks for variables that are unscoped (don't have a local variable attached).

## Why this is bad
Unscoped variables make code harder to read and debug, as well as making it harder for selene to analyze.

## Configuration
`ignore_pattern` (default: `"^_"`) - A [regular expression](https://en.wikipedia.org/wiki/Regular_expression) for variables that are allowed to be unscoped. The default allows for variables like `_` to be unscoped, as they shouldn't be used anyway.

## Example
```lua
baz = 3
```
