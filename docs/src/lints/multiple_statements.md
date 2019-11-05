# multiple_statements
## What it does
Checks for multiple statements on the same line.

## Why this is bad
This can make your code difficult to read.

## Configuration
`one_line_if` (default: `"break-return-only"`) - Defines whether or not one line if statements should be allowed. One of three options:

- "break-return-only" (default) - `if x then return end` or `if x then break end` is ok, but `if x then call() end` is not.
- "allow" - All one line if statements are allowed.
- "deny" - No one line if statements are allowed.

## Example
```lua
foo() bar() baz()
```

...should be written as...

```lua
foo()
bar()
baz()
```
