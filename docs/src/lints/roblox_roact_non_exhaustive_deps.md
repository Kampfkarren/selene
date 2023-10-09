# roblox_roact_non_exhaustive_deps
## What it does
Checks for valid dependency arrays. Verifies that dependencies are [reactive values](https://react.dev/learn/lifecycle-of-reactive-effects#effects-react-to-reactive-values) and that upvalues referenced in hooks with dependency arrays are included in the array.

## Why this is bad
Hooks that are missing dependencies will read stale values and cause bugs.

## Example
```lua
local function Component(props)
    React.useEffect(function()
        print(props)
    end) -- ok

    React.useEffect(function()
        print(props)
    end, {}) -- Missing `props`

    React.useEffect(function()
        print(props.a)
    end, { props.a }) -- ok

    React.useEffect(function()
        print(props.a)
    end, { props.a.b }) -- Missing `props.a`

    React.useEffect(function()
        print(props.a())
    end, { props.a() }) -- Too complex, extract `props.a()` to variable

    local a1 = props.a()
    React.useEffect(function()
        print(a1)
    end, { a1 }) -- now ok

    React.useEffect(function()
        print(props[a])
    end, { props[a] }) -- Too complex, extract `props[a]` to variable

    local a2 = props[a]
    React.useEffect(function()
        print(a2)
    end, { a2 }) -- now ok

    React.useEffect(function()
        print(props)
    end, helperFunction(props)) -- ok
end
```

## Remarks
1. This lint assumes either Roact or React is defined. [`undefined_variable`](./undefined_variable.md) will still lint, however.
2. This lint assumes the hook is prefixed with either `React`, `Roact`, `hooks` for legacy Roact, or a variable assigned to them.
3. This lint only takes effect for `useEffect`, `useMemo`, `useCallback`, and `useLayoutEffect`. Custom hooks that take dependency arrays will not lint.
4. This lint does not take effect if nothing is passed to the second argument of the hook.
5. This lint is only active if you are using the Roblox standard library.
6. This lint warns against complex dependency expressions like function calls and dynamic indexing. This currently false negatives with function calls without any indexing, such as `{ a() }`. 
7. This lint will ignore upvalues that are not reactive. Some examples of this are variables defined outside the component and variables that are known to be stable such as setter functions to `useState`.

## Deviations from [eslint-plugin-react-hooks/exhaustive-deps](https://www.npmjs.com/package/eslint-plugin-react-hooks)
1. ESLint requires passing in `a` for `a.b()` since js can implicitly pass `a` as `this` to `a.b`. Lua doesn't do this, so Selene allows `a.b` as dependencies.
2. ESLint complains about brackets in dependencies like `a["b"]` as being too complex. If string literals are inside the brackets and not a variable like `a[b]`, Selene recognizes that and treat it the same as `a.b`.
