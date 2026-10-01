if local player = getPlayer() then
	print(player.Name)
end

if const player = getPlayer() then
	print(player.Name)
end

if local admin = getAdmin() then
	print(admin)
elseif const guest = getGuest() then
	print(admin, guest)
else
	print(admin, guest)
end

print(player, admin, guest)

if local x: number? = getValue() then
	print(x)
end

local value = 1
if local value = value then
	print(value)
end

local const = 4
if const then
	print(const)
end

local name = if local player = getPlayer() then player.Name else player
local label = if local admin = getAdmin() then admin.title elseif const guest = getGuest() then admin or guest else guest
local typed = if local x: number = getValue() then x else 0

local callback = if local player = getPlayer()
	then function()
		return player.Name
	end
	else function()
		return player
	end
