# roblox_react_exhaustive_deps

## What it does
Checks that all dependencies used inside React/Roact hooks like `useEffect`, `useCallback`, and `useMemo` are correctly specified in the dependency array.

## Why this is bad
Missing dependencies can cause your effects or memoized values to use stale data, leading to bugs. Unnecessary dependencies can cause unnecessary re-renders or re-computations, hurting performance.

When you reference a variable inside a React hook callback, React expects you to declare it as a dependency so it knows when to re-run the effect or recompute the value. Failing to do so can lead to:

1. **Stale closures**: The callback captures the variable's value from when it was created, not its current value
2. **Missing updates**: Changes to dependencies won't trigger the effect to re-run
3. **Hard-to-debug issues**: The behavior might work initially but break when component re-renders occur in different orders

## Example

### Missing dependencies
```lua
local React = require(Packages.React)

local function Component(props)
    -- Bad: props.userId is used but not in dependencies
    React.useEffect(function()
        fetchUser(props.userId)
    end, {})
    
    -- Good: all dependencies are listed
    React.useEffect(function()
        fetchUser(props.userId)
    end, { props.userId })
end
```

### Unnecessary dependencies
```lua
-- Bad: count is not used in the effect
React.useEffect(function()
    print("Hello")
end, { count })

-- Good: empty dependencies since nothing is used
React.useEffect(function()
    print("Hello")
end, {})
```

### Property access dependencies
```lua
-- Bad: props.user.name is used but not in dependencies
React.useEffect(function()
    setTitle(props.user.name)
end, {})

-- Good: property access is correctly tracked
React.useEffect(function()
    setTitle(props.user.name)
end, { props.user.name })
```

## Supported hooks
This lint checks the following React/Roact hooks:
- `useEffect` - dependencies are the second parameter
- `useLayoutEffect` - dependencies are the second parameter  
- `useCallback` - dependencies are the second parameter
- `useMemo` - dependencies are the second parameter
- `useImperativeHandle` - dependencies are the third parameter

## Remarks
This lint works with both the legacy Roact API (using `Roact.useEffect`) and the new React-like API (using `React.useEffect`).

The lint analyzes variable references within the hook callback and compares them against the declared dependency array. It will:

1. Report missing dependencies that are used in the callback but not listed
2. Report unnecessary dependencies that are listed but not used
3. Track property access (e.g., `props.value`) as separate dependencies
4. Ignore built-in Lua globals and Roblox APIs (like `print`, `game`, `workspace`, etc.)

### Limitations
- The lint assumes variables are stable and doesn't track complex control flow
- It doesn't detect dependencies in nested function definitions
- Complex dependency expressions (computed property access with brackets) may not be analyzed correctly
- setState functions and refs are currently not detected as stable (unlike in React)

This lint is only active when using the Roblox standard library.

## Configuration
This lint does not have any configuration options.

