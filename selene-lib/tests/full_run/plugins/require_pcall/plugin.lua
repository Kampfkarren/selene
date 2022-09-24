local require_pcall = {}

require_pcall.name = "require_pcall"
require_pcall.severity = "warning"

function require_pcall.pass(ast, context)
	local pcallRanges = {}

	ast:visit({
		FunctionCall = function(functionCall)
			local name = functionCall.prefix:match("Name")
			if name == nil then
				return
			end

			if name.token:print() ~= "pcall" then
				return
			end

			table.insert(pcallRanges, { functionCall:range() })
		end,
	})

	ast:visit({
		FunctionCall = function(functionCall)
			local functionCallName = purge_trivia(functionCall.prefix):print()
			local global = context.standard_library:find_global(functionCallName)
			if global == nil then
				return
			end

			if not global.extra.require_pcall then
				return
			end

			local rangeStart, rangeEnd = functionCall:range()

			for _, pcallRange in pcallRanges do
				if pcallRange[1] <= rangeStart and rangeEnd <= pcallRange[2] then
					return
				end
			end

			lint(string.format("`%s` must be wrapped in a pcall", functionCallName), functionCall.prefix)
		end,
	})
end

return require_pcall
