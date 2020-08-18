# bad_string_escape
## What it does
Checks for invalid, malformed, or unnecessary string escape sequences.

## Why this is bad
Invalid string escapes don't do anything, so should obviously be caught in dealt with. Additionally, in double strings, you shouldn't escape single quote strings since it makes the string less readable. Same with single quote strings and double quotes.

In some cases (specifically `\x` and `\u` in a Roblox codebase) it's possible to write an escape sequence that looks right but doesn't work when ran. Because this is probably not intentional, they are caught by this lint.

## Example
```lua
print("\m") -- This escape sequence is invalid.

print("don\'t") -- This escape makes the string less readable than `don't`

print('\"foo\"') -- This escape makes the string less readable than `"foo"`
```

In Roblox:
```lua
print("\x1") -- This escape sequence is malformed (\x expects two hex digits after it)

print("\u{1234") -- This escape sequence is *also* malformed (\u needs a closing bracket)

print("\u{110000}") -- This escape sequence is invalid because the max codepoint passed to \u is `10ffff`.
```