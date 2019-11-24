# incorrect_standard_library_use
## What it does
Checks for correct use of [the standard library](../cli/std.md).

## Example
```lua
for _, shop in pairs(GoldShop, ItemShop, MedicineShop) do
```

## Remarks
**It is highly recommended that you do not turn this lint off.** If you are having standard library issues, modify your standard library instead to be correct. If it is a problem with an official standard library (Ex: the Lua 5.1 or Roblox ones), you can file an [issue on GitHub](https://github.com/Kampfkarren/selene/issues).
