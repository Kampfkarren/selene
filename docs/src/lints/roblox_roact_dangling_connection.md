# roblox_roact_dangling_connection
## What it does
Checks for connections in Roact components that are either not assigned to a variable or passed as arguments.

## Why this is bad
This indicates a memory leak and can cause unexpected behaviors.

## Example
```lua
local function MyComponent()
    useEffect(function()
        a:Connect()
    end, {})
end
```

## Remarks
This lint is active if the file has a variable named `Roact` or `React` and that the connection is made within a function.

This checks for connections by identifying the following keywords:
* Connect
* connect
* ConnectParallel
* Once
