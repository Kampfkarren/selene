-- Good: lowercase variable names with underscores
local good_var = 10
local x = 5
local my_number = 42
local foo123 = 100

-- Bad: variable names with uppercase letters
local BadVar = 10
local mixedCase = 20

-- Good: function names (lowercase, optional leading underscore)
function doThing()
    return 1
end

function _private()
    return 2
end

local function myFunc()
    return 3
end

-- Bad: function names (uppercase or mixed case)
function DoThing()
    return 1
end

function UPPERCASE_FUNCTION()
    return 2
end

local function PrivateFunc()
    return 3
end

-- Good: PascalCase names (detected as types by heuristic)
local MyClass = {}
local UserData = {}

-- Bad: snake_case with underscores (detected as invalid type names)
local my_class = {}
local user_data = {}
