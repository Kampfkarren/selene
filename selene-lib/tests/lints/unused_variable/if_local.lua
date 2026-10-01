if local unusedPlayer = getPlayer() then
	print("unused")
end

if local usedPlayer = getPlayer() then
	print(usedPlayer)
elseif const unusedGuest = getGuest() then
	print("unused")
end

local value = if local unusedX = getValue() then 1 else 2
local other = if local usedX = getValue() then usedX else 2
print(value, other)
