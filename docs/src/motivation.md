# Motivation

### Because bugs
When writing any code, it's very easy to make silly mistakes that end up introducing bugs. A lot of the time, these bugs are hard to track down and debug, and sometimes are even harder to replicate.

This risk is made ever more real because of the generally lax nature of Lua. Incorrect code is regularly passed off and isn't noticed until something breaks at runtime. Sometimes you'll get a clear error message, and will have to spend time going back, fixing the code, and making sure you actually fixed it. Other times, the effects are more hidden, and instead of getting an error your code will just pass through along, in an incorrect state.

Take, for example, this code:

```lua
function Player:SwapWeapons()
    self.CurrentWeapon = self.SideWeapon
    self.SideWeapon = self.CurrentWeapon
end
```

This is code that is technically correct, but is absolutely not what you wanted to write. However, because it is *technically* correct, Lua will do exactly what you tell it to do, and so...

- Player wants to swap their weapons
- Your code calls `player:SwapWeapons()`
- Their current weapon is set to their side weapon...
- ...but their side weapon is set to their current weapon afterwards, which is what they just equipped!

Uh oh! After debugging this, you realize that you actually meant to write was...

```lua
function Player:SwapWeapons()
    self.CurrentWeapon, self.SideWeapon = self.SideWeapon, self.CurrentWeapon
end
```

If you were using selene, you would've been alerted right away that your original code looked like an [`almost_swapped`](../lints/almost_swapped.md).

```
error[almost_swapped]: this looks like you are trying to swap `self.CurrentWeapon` and `self.SideWeapon`

   ┌── fail.lua:4:5 ───
   │
 4 │ ╭     self.CurrentWeapon = self.SideWeapon
 5 │ │     self.SideWeapon = self.CurrentWeapon
   │ ╰────────────────────────────────────────^
   │
   = try: `self.CurrentWeapon, self.SideWeapon = self.SideWeapon, self.CurrentWeapon`
```

Other bugs arise because of Lua's lack of typing. While it can feel freeing to developers to not have to specify types everywhere, it makes it easier to mess up and write broken code. For example, take the following code:

```lua
for _, shop in pairs(GoldShop, ItemShop, MedicineShop) do
```

This code is yet again technically correct, but not what we wanted to do. `pairs` will take the first argument, `GoldShop`, and ignore the rest. Worse, the `shop` variable will now be the values of the contents of `GoldShop`, not the shop itself. This can cause massive headaches, since although you're likely to get an error later down the line, it's more likely it'll be in the vein of "attempt to index a nil value `items`" than something more helpful. If you used `ipairs` instead of `pairs`, your code inside might just not run, and won't produce an error at all.

Yet again, selene saves us.

```
error[incorrect_standard_library_use]: standard library function `pairs` requires 1 parameters, 3 passed

   ┌── fail.lua:1:16 ───
   │
 1 │ for _, shop in pairs(GoldShop, ItemShop, MedicineShop) do
   │                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   │
```

This clues the developer into writing the code they meant to write:
```lua
for _, shop in pairs({ GoldShop, ItemShop, MedicineShop }) do
```

### Because idiomatic Lua
While it's nice to write code however you want to, issues can arise when you are working with other people, or plan on open sourcing your work for others to contribute to. It's best for everyone involved if they stuck to the same way of writing Lua.

Consider this contrived example:

```lua
call(1 / 0)
```

The person who wrote this code might have known that `1 / 0` evaluates to `math.huge`. However, anyone working on that code will likely see it and spend some time figuring out why they wrote the code that way.

If the developer was using selene, this code would be denied:

```
warning[divide_by_zero]: dividing by zero is not allowed, use math.huge instead

   ┌── fail.lua:1:6 ───
   │
 1 │ call(1 / 0)
   │      ^^^^^
   │
```

Furthermore, selene is meant to be easy for developers to add their own lints to. You could create your own lints for your team to prevent behavior that is non-idiomatic to the codebase. For example, let's say you're working on a [Roblox](https://developer.roblox.com/en-us) codebase, and you don't want your developers using the data storage methods directly. You could create your own lint so that this code:

```lua
local DataStoreService = game:GetService("DataStoreService")
```

...creates a warning, discouraging its use. For more information on how to create your own lints, check out the [contribution guide](../contributing.md).
