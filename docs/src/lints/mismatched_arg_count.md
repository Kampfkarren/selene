# mismatched_arg_count
## What it does
Checks for too many arguments passed to function calls of defined functions.

## Why this is bad
These arguments provided are unnecessary, and can indicate that the function definition is not what was expected.

## Example
```lua
local function foo(a, b)
end

foo(1, 2, 3) -- error, function takes 2 arguments, but 3 were supplied
```

## Remarks
This lint does not handle too few arguments being passed, as this is commonly inferred as passing `nil`. For example,
`foo(1)` could be used when meaning `foo(1, nil)`.

If a defined function is reassigned anywhere in the program, it will not trigger the lint. This is because static analysis can no longer tell
which definition is the right one in the given context. Take the example:
```lua
local function foo(a, b, c)
    print("a")
end

function updateFoo()
    foo = function(a, b, c, d)
        print("b")
    end
end

foo(1, 2, 3, 4) --> "a" [mismatched args, but selene doesn't know]
updateFoo()
foo(1, 2, 3, 4) --> "b" [no more mismatched args]
```
selene can not tell that `foo` corresponds to a new definition because `updateFoo()` was called in the current context, without actually *running* the program.
