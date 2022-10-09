# Filtering
Lints can be toggled on and off in the middle of code when necessary through the use of special comments.

## Allowing/denying lints for a piece of code
Suppose we have the following code:

```lua
local something = 1
```

selene will correctly attribute this as an unused variable:

```
warning[unused_variable]: something is assigned a value, but never used

   ┌── code.lua:1:7 ───
   │
 1 │ local something = 1
   │       ^^^^^^^^^
   │
```

However, perhaps we as the programmer have some reason for leaving this unused (and not renaming it to `_something`). This would be where inline lint filtering comes into play. In this case, we would simply write:

```lua
-- selene: allow(unused_variable)
local something = 1
```

This also works with settings other than `allow`--you can `warn` or `deny` lints in the same fashion. For example, you can have a project with the following `selene.toml` [configuration](./configuration.md):

```toml
[lints]
unused_variable = "allow" # I'm fine with unused variables in code
```

...and have this in a separate file:

```lua
-- I'm usually okay with unused variables, but not this one
-- selene: deny(unused_variable)
local something = 1
```

This is applied to the entire piece of code its near, *not* just the next line. For example:

```lua
-- selene: allow(unused_variable)
do
    local foo = 1
    local bar = 2
end
```

...will silence the unused variable warning for both `foo` and `bar`.

## Allowing/denying lints for an entire file
If you want to allow/deny a lint for an entire file, you can do this by attaching the following code to the beginning:

```lua
--# selene: allow(lint_name)
```

The `#` tells selene that you want to apply these globally.

These *must* be before any code, otherwise selene will deny it. For example, the following code:

```lua
local x = 1
--# selene: allow(unused_variable)
```

...will cause selene to error:

```
warning[unused_variable]: x is assigned a value, but never used
  ┌─ -:1:7
  │
1 │ local x = 1
  │       ^

error[invalid_lint_filter]: global filters must come before any code
  ┌─ -:1:1
  │
1 │ local x = 1
  │ ----------- global filter must be before this
2 │ --# selene: allow(unused_variable)
```

## Combining multiple lints

You can filter multiple lints in two ways:
```lua
-- selene: allow(lint_one)
-- selene: allow(lint_two)

-- or...

-- selene: allow(lint_one, lint_two)
```
