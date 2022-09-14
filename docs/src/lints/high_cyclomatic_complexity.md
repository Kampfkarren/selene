# high_cyclomatic_complexity
## What it does
Measures the cyclomatic complexity of a function to see if it exceeds the configure maximum.

## Why this is bad
High branch complexity can lead to functions that are hard to test, and harder to reason about.

## Example
```lua
return function(props)
    return React.createElement("TextLabel", { style = if props.blue then 0 else 1 },{
        Child1 = props.mask and React.createElement("Instance") or nil,
        React.createElement(if _G.__PROFILING__ then "TextLabel" elseif _G.__DEV__ then "Instance" else "Non")
    })({
        if props == nil then "bob" else "alice"
    })
    (function()
        if _G.__DEV__ then
          print("howdy")
        end
    end)
end
```

## Remarks

This lint is off by default. In order to enable it, add this to your selene.toml:

```toml
[rules]
high_cyclomatic_complexity = "warn" # Or "deny"
```
