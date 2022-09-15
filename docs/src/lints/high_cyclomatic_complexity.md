# high_cyclomatic_complexity
## What it does
Measures the [cyclomatic complexity](https://en.wikipedia.org/wiki/Cyclomatic_complexity) of a function to see if it exceeds the configure maximum.

## Why this is bad
High branch complexity can lead to functions that are hard to test, and harder to reason about.

## Configuration
`maximum_complexity` (default: `40`) - A number that determines the maximum threshold for cyclomatic complexity, beyond which the lint will report.

## Example
```lua
function MyComponent(props)
    if props.option1 == "enum_value1" then          -- 1st path
        return React.createElement("Instance")
    elseif props.option1 == "enum_value2"           -- 2nd path
      or props.option2 == nil then                  -- 3rd path
        return React.createElement(
          "TextLabel",
          { Text = if _G.__DEV__ then "X" else "Y" }-- 4th path
        )
    else
        return if props.option2 == true             -- 5th path
          then React.createElement("Frame")
          else nil
    end
end
```

## Remarks

This lint is off by default. In order to enable it, add this to your selene.toml:

```toml
[rules]
high_cyclomatic_complexity = "warn" # Or "deny"
```
