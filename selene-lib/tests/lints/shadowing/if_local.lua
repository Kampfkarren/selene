local player = getPlayer()

if local player = getPlayer() then
	print(player)
end

if local guest = getGuest() then
	print(guest)
elseif local guest = getOtherGuest() then
	print(guest)
end
