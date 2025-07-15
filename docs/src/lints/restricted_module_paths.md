# restricted_module_paths

## What it does

Checks for restricted module paths in any expression context, preventing usage of specific module paths wherever they appear in code.

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
local deprecatedFunction = OldLibrary.Utils.deprecatedFunction

OldLibrary.Utils.deprecatedFunction()

fn(OldLibrary.Utils.deprecatedFunction)

local config = { callback = OldLibrary.Utils.deprecatedFunction }

function getHandler()
    return OldLibrary.Utils.deprecatedFunction
end

local nested = { deep = { handler = OldLibrary.Utils.deprecatedFunction } }

local handler = condition and OldLibrary.Utils.deprecatedFunction or nil
```

## Remarks

This lint comprehensively checks for restricted module paths in:
- **Assignments**: `local deprecatedFunction = OldLibrary.Utils.deprecatedFunction`
- **Function calls**: `OldLibrary.Utils.deprecatedFunction()`
- **Function arguments**: `fn(OldLibrary.Utils.deprecatedFunction)`
- **Table constructors**: `local config = { callback = OldLibrary.Utils.deprecatedFunction }`
- **Return statements**: `return OldLibrary.Utils.deprecatedFunction`
- **Nested table structures**: `local nested = { deep = { handler = OldLibrary.Utils.deprecatedFunction } }`
- **Conditional expressions**: `local handler = condition and OldLibrary.Utils.deprecatedFunction or nil`

It does not check:
- **String require statements**: `require("Module.SubModule")`
- **String literals**: `"Module.SubModule.function"`

The lint performs exact string matching on the full module path, so `"OldLibrary.Utils.deprecatedFunction"` will match exactly but not `"OldLibrary.Utils.deprecatedFunctionExtended"`.
