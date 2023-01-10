-- `manual_table_clone` changes its range if anything is in between the definition and loop,
-- but the lint filter should always be the same.

local stuff = {}

local output = {}

-- selene: allow(manual_table_clone)
for key, value in pairs(stuff) do
	output[key] = value
end

local other = {}
for key, value in pairs(stuff) do
	other[key] = value
end
