local function falsePositive1(...)
	local result = {}

	for i = 1, select("#", ...) do
		local dictionary = select(i, ...)
		for key, value in pairs(dictionary) do
			result[key] = value
		end
	end

	return result
end

local function falsePositive2(t)
	local result = {}
	local count = 0

	while count < 20 do
		count = count + 1
		for key, value in pairs(t) do
			result[key] = value
		end
	end

	return result
end

local function falsePositive3(t)
	local result = {}
	local count = 0

	repeat
		count = count + 1
		for key, value in pairs(t) do
			result[key] = value
		end
	until count > 20

	return result
end

local result4 = {}
local function falsePositive4(t)
    for key, value in t do
        result3[key] = value
    end
end

local result5 = {}
local function falsePositive5(t)
	local function f() end

    for key, value in t do
        result4[key] = value
    end
end

local function falsePositive6(t)
    local result = {}

	if b then return end

    if a then
        for key, value in t do
            result4[key] = value
        end
    end
end

local function notFalsePositive1(t)
	local result = {}

	for i = 1, 10 do
		print(i)
	end

	for key, value in pairs(t) do
		result[key] = value
	end

	return result
end

local function notFalsePositive2(t)
	for i = 1, 10 do
		local result = {}

		for key, value in pairs(t) do
			result[key] = value
		end
	end
end
