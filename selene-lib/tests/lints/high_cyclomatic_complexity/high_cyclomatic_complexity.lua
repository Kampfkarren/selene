local React = {
    createElement = function(...) end
}
local function withStyle(x) return x end

return withStyle(function(props)
        return React.createElement("TextLabel", { style = if props.blue then 0 else 1 },{
            Child1 = props.mask and React.createElement("Instance") or nil,
            React.createElement(if _G.__PROFILING__ then "TextLabel" elseif _G.__DEV__ then "Instance" else "Non"),
            React.createElement(if _G.__PROFILING__ then "TextLabel" elseif _G.__DEV__ then "Instance" else "Non"),
            React.createElement(if _G.__PROFILING__ then "TextLabel" elseif _G.__DEV__ then "Instance" else "Non"),
            React.createElement(if _G.__PROFILING__ then "TextLabel" elseif _G.__DEV__ then "Instance" else "Non"),
            React.createElement(if _G.__PROFILING__ then "TextLabel" elseif _G.__DEV__ then "Instance" else "Non"),
            React.createElement(if _G.__PROFILING__ then "TextLabel" elseif _G.__DEV__ then "Instance" else "Non"),
            React.createElement(if _G.__PROFILING__ then "TextLabel" elseif _G.__DEV__ then "Instance" else "Non"),
            React.createElement(if _G.__PROFILING__ then "TextLabel" elseif _G.__DEV__ then "Instance" else "Non"),
            React.createElement(if _G.__PROFILING__ then "TextLabel" elseif _G.__DEV__ then "Instance" else "Non"),
            React.createElement(if _G.__PROFILING__ then "TextLabel" elseif _G.__DEV__ then "Instance" else "Non"),
            React.createElement(if _G.__PROFILING__ then "TextLabel" elseif _G.__DEV__ then "Instance" else "Non"),
            React.createElement(if _G.__PROFILING__ then "TextLabel" elseif _G.__DEV__ then "Instance" else "Non"),
            React.createElement(if _G.__PROFILING__ then "TextLabel" elseif _G.__DEV__ then "Instance" else "Non"),
            React.createElement(if _G.__PROFILING__ then "TextLabel" elseif _G.__DEV__ then "Instance" else "Non"),
            React.createElement(if _G.__PROFILING__ then "TextLabel" elseif _G.__DEV__ then "Instance" else "Non"),
            React.createElement(if _G.__PROFILING__ then "TextLabel" elseif _G.__DEV__ then "Instance" else "Non"),
            React.createElement(if _G.__PROFILING__ then "TextLabel" elseif _G.__DEV__ then "Instance" else "Non"),
            React.createElement(if _G.__PROFILING__ then "TextLabel" elseif _G.__DEV__ then "Instance" else "Non"),
            React.createElement(if _G.__PROFILING__ then "TextLabel" elseif _G.__DEV__ then "Instance" else "Non"),
            React.createElement(if _G.__PROFILING__ then "TextLabel" elseif _G.__DEV__ then "Instance" else "Non"),
            React.createElement(if _G.__PROFILING__ then "TextLabel" elseif _G.__DEV__ then "Instance" else "Non"),
            React.createElement(if _G.__PROFILING__ then "TextLabel" elseif _G.__DEV__ then "Instance" else "Non"),
        })({
            if props == nil then "mallet" else "alice"
        })
    end)(function()
        if _G.__DEV__ then
            print("howdy")
        end
    end)()
