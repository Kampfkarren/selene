# restricted_imports

## What it does

Checks for restricted import paths in local assignments and prevents usage of specific module paths.

## Why this is bad

Some module paths may be deprecated, internal-only, or have better alternatives that should be used instead. This lint helps enforce coding standards and prevents usage of restricted APIs.

## Configuration

`restricted_paths` - A map of restricted import paths to their respective error messages.

```toml
[config.restricted_imports.restricted_paths]
"OldLibrary.Utils.deprecatedFunction" = "OldLibrary.Utils.deprecatedFunction has been deprecated. Use NewLibrary.Utils.modernFunction instead."
```

## Example

```lua
local deprecatedFunction = OldLibrary.Utils.deprecatedFunction
```

## Remarks

This lint only checks local assignments with property access patterns (e.g., `local x = Module.SubModule.function`). It does not check:
- Function calls like `Module.SubModule.function()`
- Global assignments like `x = Module.SubModule.function`
- Require statements like `require("Module.SubModule")`

For broader restriction of tokens regardless of context, consider using the [`denylist_filter`](./denylist_filter.md) lint instead.
