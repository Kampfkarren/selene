# roblox_incorrect_color3_new_bounds
## What it does
Checks for uses of `Color3.new` where the arguments are not between 0 and 1.

## Why this is bad
Most likely, you are trying to use values of 0 to 255. This will not give you an error, and will silently give you the wrong color. You probably meant to use `Color3.fromRGB` instead.

## Example
```lua
Color3.new(255, 0, 0)
```

## Remarks
This lint is only active if you are using the Roblox standard library.
