# restricted_module_paths

## What it does

Checks for restricted module paths in local assignments and function calls, preventing usage of specific module paths.

## Why this is bad

Some module paths may be deprecated, internal-only, or have better alternatives that should be used instead. This lint helps enforce coding standards and prevents usage of restricted APIs.

## Configuration

`restricted_paths` - A map of restricted module paths to their respective error messages.

```toml
[config.restricted_module_paths.restricted_paths]
"OldLibrary.Utils.deprecatedFunction" = "OldLibrary.Utils.deprecatedFunction has been deprecated. Use NewLibrary.Utils.modernFunction instead."
```

## Example

```lua
-- This will trigger the lint (local assignment)
local deprecatedFunction = OldLibrary.Utils.deprecatedFunction

-- This will also trigger the lint (function call)
OldLibrary.Utils.deprecatedFunction()
```

## Remarks

This lint checks:
- Local assignments with property access patterns (e.g., `local x = Module.SubModule.function`)
- Function calls with module paths (e.g., `Module.SubModule.function()`)

It does not check:
- Global assignments like `x = Module.SubModule.function`
- Require statements like `require("Module.SubModule")`
- String literals containing module paths

For broader restriction of tokens regardless of context, consider using the [`denylist_filter`](./denylist_filter.md) lint instead.
