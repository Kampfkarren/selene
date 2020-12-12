# type_check_inside_call
## What it does
Checks for `type(foo == "type")`, instead of `type(foo) == "type"`.

## Why this is bad
This will always return `"boolean"`, and is undoubtedly not what you intended to write.

## Example
```lua
return type(foo == "number")
```

...should be written as...

```lua
return type(foo) == "number"
```

## Remarks
When using the Roblox standard library, this checks `typeof` as well.
