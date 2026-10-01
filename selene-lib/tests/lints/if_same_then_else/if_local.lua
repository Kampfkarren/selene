if local admin = getAdmin() then
	print(admin)
elseif local admin = getModerator() then
	print(admin)
end

if local admin = getAdmin() then
	print(admin)
elseif foo then
	print(admin)
end

if foo then
	print(admin)
elseif local admin = getAdmin() then
	print(admin)
elseif bar then
	print(admin)
end
