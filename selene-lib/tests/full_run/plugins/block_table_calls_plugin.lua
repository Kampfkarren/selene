local block_table_calls = {}

block_table_calls.name = "block_table_calls"
block_table_calls.severity = "warning"

function block_table_calls.pass(ast)
	ast:visit({
		FunctionArgs = function(function_args)
			if function_args.kind == "TableConstructor" then
				lint("table calls are not allowed", function_args)
			end
		end,
	})
end

return block_table_calls
