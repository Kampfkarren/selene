# duplicate_condition
## What it does
Checks for conditions that can be simplified down.

## Why this is bad
This generally indicates a bug in something like the fake ternary idiom (`x and y or z`).

## Example
```lua
print(x and x)
```

...should be written as...
```lua
print(x)
```

## Remarks
Does not attempt to solve conditionals that have potential side effects. For example, the following code:

```lua
print(call() or call())
```

...could potentially be correct if `call` is non-deterministic.

A caveat to this caveat, however, is that it expects that any potential metamethods you implement are sane. For instance, `x.y or x.y` can potentially *not* simplify down to `x.y`, but only if you implement a non-idiomatic `__index`. duplicate_condition however, will still suggest it.

This lint also does not correctly identify truthiness/falsiness outside of `true`, `false`, and `nil`. That means the following code:

```lua
print(3 or 2)
```

...does not lint, as `3` and `2` are separate terms.
