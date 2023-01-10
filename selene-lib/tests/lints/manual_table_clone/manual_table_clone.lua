local new1 = {}
for key, value in pairs(stuff) do
	new1[key] = value -- fail
end

local new2 = {}
for key, value in ipairs(stuff) do
	new2[key] = value -- fail
end

local new3 = {}
for key, value in next, stuff do
	new3[key] = value -- fail
end

local new4 = {}
for key, value in stuff do
	new4[key] = value -- fail
end

local new5 = {}
for key, value in pairs(stuff) do
	if key == "foo" then
		new5[key] = value -- pass
	end
end

local new6 = {}
new6.used = "welp"
for key, value in pairs(stuff) do
	new6[key] = value -- pass
end

local new7 = {}
for key, value in pairs(stuff) do
	new7[key] = value -- fail
end
new7.used = "too late"

local new8 = {}
for key, value in pairs(getStuff()) do
	new8[key] = value -- fail
end

local new9 = {}

-- For Luau, this can be treated the same as looping over a table result
for key, value in what(stuff) do
	new9[key] = value -- fail
end

local new10 = {}
for key, value in pairs(stuff), what(stuff) do
	new10[key] = value -- shrug
end

local new11 = {}
for key, value in what(stuff), pairs(stuff) do
	new11[key] = value -- shrug
end

local new12 = {}
for key, value in pairs(what)(the) do
	new12[key] = value -- shrug
end

for key, value in pairs(stuff) do
	no()[key] = value -- pass
end

for key, value in pairs(stuff) do
	too.bad[key] = value -- pass
end

for key, value in pairs(stuff) do
	global[key] = value -- pass
end

local new13, new14 = {}, {}
for key, value in pairs(stuff) do
	new13[key], new14[key] = value, -value -- pass
end

local new15 = { x = 1 }
for key, value in pairs(stuff) do
	new15[key] = value -- pass
end

local new16 = {}
whoKnows(new16)
for key, value in pairs(stuff) do
	new16[key] = value -- pass
end

local new17 = whoKnows()
for key, value in pairs(stuff) do
	new17[key] = value -- pass
end

local new18 = {}
for key, value in what, stuff do
	new18[key] = value -- pass
end

local new19 = {}
blaBlaBla()
someStuffHere()
for key, value in pairs(stuff) do
	new19[key] = value -- fail
end

-- weird but valid
local function ipairs(_) end
local newWeirdIpairs = {}
for key, value in ipairs(stuff) do
	newWeirdIpairs[key] = value -- fail, but don't report as ipairs
end
