# Luacheck Comparison

## selene vs. luacheck
selene is not the first Lua linter. The main inspiration behind selene is [luacheck](https://luacheck.readthedocs.io/en/stable/). However, the two have very little in common besides inception.

- selene is actively maintained, while at the time of writing luacheck's last commit was in October 2018.
- selene is written in Rust, while luacheck is written in Lua. In practice, this means that selene is much faster than luacheck while also being able to easily take advantage of features luacheck cannot because of the difficulty of using dependencies in Lua.
- selene is multithreaded, again leading to significantly better performance.
- selene has rich output, while luacheck has basic output.

selene:

```
error[suspicious_reverse_loop]: this loop will only ever run once at most

   ┌── fail.lua:1:9 ───
   │
 1 │ for _ = #x, 1 do
   │         ^^^^^
   │
   = help: try adding `, -1` after `1`
```

luacheck:

```
Checking fail.lua                                 2 warnings

    fail.lua:1:1: numeric for loop goes from #(expr) down to 1 but loop step is not negative
```

- selene uses [TOML](https://github.com/toml-lang/toml) files for configuration, while luacheck uses `.luacheckrc`, which runs Lua.
- selene allows for [standard library configuration](./usage/std.md) such as argument types, argument counts, etc, while luacheck only allows knowing that fields exist and can be written to. In practice, this means that selene catches:

```lua
for _, shop in pairs(GoldShop, ItemShop, MedicineShop) do

math.pi()
```

...while luacheck does not.
- selene has English names for lints instead of arbitrary numbers. In luacheck, you ignore "[`211`](https://luacheck.readthedocs.io/en/stable/warnings.html#unbalanced-assignments)", while in selene you ignore "[`unbalanced_assignments`](./lints/unbalanced_assignments.md)".
- selene has distinctions for "deny" and "warn", while every luacheck lint is the same.
- selene has a much simpler codebase, and is much easier to add your own lints to.
- selene has optional support and large focus specifically for Roblox development.
- selene will only show you files that lint, luacheck only does this with the `-q` option (quiet).
- selene [filters specific lints](./usage/filtering.md) and applies over code rather than lines, luacheck does not.
- selene has [significantly more lints](./lints/index.md).

This is not to say selene is objectively better than luacheck, at least not yet.
- luacheck has lints for long lines and whitespace issues, selene does not as it is unclear whether style issues like these are fit for a linter or better under the scope of a Lua beautifier.
- luacheck officially supports versions past Lua 5.1, selene does not yet as there is not much demand.
- luacheck supports the following lints that selene does not yet:
  - Unreachable code
  - Unused labels (selene does not officially support Lua 5.2 yet)
  - Detecting variables that are only ever mutated, but not read
  - Using uninitialized variables

## Migration
luacheck does not require much configuration to begin with, so migration should be easy.

- You can configure what lints are allowed in the [configuration](./usage/configuration.md#changing-the-severity-of-lints).
- Do you have a custom standard library (custom globals, functions, etc)? Read the [standard library guide](./usage/std.md).
  - Are you a Roblox developer using something like [luacheck-roblox](https://github.com/Quenty/luacheck-roblox/)? A featureful standard library for Roblox is generated with every commit on GitHub. TODO: Have a flag in the selene CLI to generate a Roblox standard library a la `generate-roblox-std`? Should `generate-roblox-std` be uploaded to crates.io?
