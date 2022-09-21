local cond
local h = function()
    repeat
        print(2)
     until cond()
end

function cond()
    return _G.__PROFILE__
end
return h