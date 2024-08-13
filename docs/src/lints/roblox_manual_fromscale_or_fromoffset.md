# roblox_manual_fromscale_or_fromoffset
## What it does
Checks for uses of `UDim2.new` where the arguments could be simplified to `UDim2.fromScale` or `UDim2.fromOffset`.

## Why this is bad
This reduces readability of `UDim2.new()` construction.

## Example
```lua
UDim2.new(1, 0, 1, 0)
```

## Remarks
This lint is only active if you are using the Roblox standard library.
