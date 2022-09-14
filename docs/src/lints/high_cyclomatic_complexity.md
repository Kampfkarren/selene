# high_cyclomatic_complexity
## What it does
Measures the cyclomatic complexity of a function to see if it exceeds the configure maximum.

## Why this is bad
High branch complexity can lead to functions that are hard to test, and harder to reason about.

## Example
```lua
function MyComponent(props)
    if(props.option1 == "enum_value1" then          -- 1st path
        return React.createElement("Instance")
    else if props.option1 == "enum_value2"          -- 2nd path
      or props.option2 == nil then                  -- 3rd path
        return React.createElement(
          "TextLabel",
          nil,
          { text = if _G.__DEV__ then "X" else "Y" }-- 4th path
        )
    else
        return if props.option2 == true             -- 5th path
          then React.createElement("Frame")
          else nil
    end
end
```
