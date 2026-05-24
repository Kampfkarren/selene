-- valid function names (fn_re = ^_?[a-z][a-zA-Z0-9]*$)
function doThing() end
local function _helper() end

-- invalid function (PascalCase)
function DoThing() end
