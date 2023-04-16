# roblox_suspicious_udim2_new
## What it does
Checks for too little arguments passed to `UDim2.new()`.

## Why this is bad
Passing in an incorrect number of arguments can indicate that the user meant to use `UDim2.fromScale` or `UDim2.fromOffset`.
Even if the user really only needed to pass in a fewer number of arguments to `UDim2.new`, this lowers readability
as it calls into question whether it's a bug or if the user truly meant it to use `UDim2.new`.

## Example
```lua
UDim2.new(1, 1) -- error, UDim2.new takes 4 numbers, but 2 were provided.
```

## Remarks
This lint is only active if you are using the Roblox standard library.
