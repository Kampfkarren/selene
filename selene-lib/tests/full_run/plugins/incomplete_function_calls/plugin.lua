-- A demo of how to use ScopeManager
local incomplete_function_calls = {}

incomplete_function_calls.name = "incomplete_function_calls"
incomplete_function_calls.severity = "warning"

function incomplete_function_calls.pass(ast, context)
	local scopeManager = context.scope_manager

	local functions = {}

	-- First, learn all functions
	ast:visit({
		LocalFunction = function(localAssignment)
			local variable = scopeManager:variable_at_byte(localAssignment.name:range())

			functions[tostring(variable.id)] = #localAssignment.body.parameters
		end,
	})

	-- Now that we've mapped out all functions, then start to check calls
	ast:visit({
		FunctionCall = function(function_call)
			local name = function_call.prefix:match("Name")
			if name == nil then
				return
			end

			local reference = scopeManager:reference_at_byte(name:range())
			local resolvedReference = scopeManager:resolve_reference(reference)

			if resolvedReference == nil then
				return
			end

			local expectedArgCount = functions[tostring(resolvedReference.id)]
			print("expectedArgCount = ", expectedArgCount)
			if expectedArgCount == nil then
				return
			end

			local call = function_call.suffixes[#function_call.suffixes]
				:expect("Call")
				:expect("AnonymousCall")
				:match("Parentheses")

			if call == nil then
				return
			end

			if #call.arguments < expectedArgCount then
				lint(
					string.format("not enough arguments. expected %d, got %d", expectedArgCount, #call.arguments),
					function_call
				)
			end
		end,
	})
end

return incomplete_function_calls
