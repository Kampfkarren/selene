-- Test local assignment restriction
local deprecatedFunction = OldLibrary.Utils.deprecatedFunction

-- Test function call restriction
OldLibrary.Utils.deprecatedFunction()

-- Test function argument restriction
fn(OldLibrary.Utils.deprecatedFunction)

-- Test table constructor restriction
local config = { callback = OldLibrary.Utils.deprecatedFunction }

-- Test return statement restriction
function getHandler()
    return OldLibrary.Utils.deprecatedFunction
end

-- Test nested table restriction
local nested = { deep = { handler = OldLibrary.Utils.deprecatedFunction } }

-- Test if expression restriction
local handler = condition and OldLibrary.Utils.deprecatedFunction or nil
