# Standard Library Format
selene provides a robust standard library format to allow for use with environments other than vanilla Lua. Standard libraries are defined in the form of [TOML](https://github.com/toml-lang/toml) files.

# Examples
For examples of the standard library format, see:
- [`lua51.toml`](https://github.com/Kampfkarren/selene/blob/master/selene-lib/default_std/lua51.toml) - The default standard library for Lua 5.1
- [`lua52.toml`](https://github.com/Kampfkarren/selene/blob/master/selene-lib/default_std/lua52.toml) - A standard library for Lua 5.2's additions and removals. Reference this if your standard library is based off another (it most likely is).
- [`roblox.toml`](https://gist.github.com/evaera/13c96302d308c7a9ffb2a3fc5d28ac96) - A standard library for Roblox that incorporates all the advanced features of the format. If you are a Roblox developer, don't use this as anything other than reference--an up to date version of this library is available with every commit.

## [selene]
Anything under the key `[selene]` is used for meta information. The following paths are accepted:

`[selene.base]` - Used for specifying what standard library to be based off of. Currently only accepts built in standard libraries, meaning `lua51` or `lua52`.

`[selene.name]` - Used for specifying the name of the standard library. Used internally for cases such as only giving Roblox lints if the standard library is named `"roblox"`.

`[selene.structs]` - Used for declaring [structs](#structs).

## [globals]
This is where the magic happens. The `globals` field is a dictionary where the keys are the globals you want to define. The value you give tells selene what the value can be, do, and provide.

If your standard library is based off another, overriding something defined there will use your implementation over the original.

## Any
Example:
```toml
[foo]
any = true
```

Specifies that the field can be used in any possible way, meaning that `foo.x`, `foo:y()`, etc will all validate.

## Functions
Example:

```toml
[[tonumber.args]]
type = "any"

[[tonumber.args]]
type = "number"
required = false
```

A field is a function if it contains an `args` and/or `method` field.

If `method` is specified as `true` and the function is inside a table, then it will require the function be called in the form of `Table:FunctionName()`, instead of `Table.FunctionName()`.

`args` is an array of arguments, in order of how they're used in the function. An argument is in the form of:

```
required?: false | true | string;
type: "any" | "bool" | "function" | "nil"
    | "number" | "string" | "table" | "..."
    | string[] | { "display": string }
```

## "required"
- `true` - The default, this argument is required.
- `false` - This argument is optional.
- A string - This argument is required, and not using it will give this as the reason why.

## Argument types
- `"any"` - Allows any value.
- `"bool"`, `"function"`, `"nil"`, `"number"`, `"string"`, `"table"` - Expects a value of the respective type.
- `"..."` - Allows any number of variables after this one. If `required` is true (it is by default), then this will lint if no additional arguments are given. It is incorrect to have this in the middle.
- Constant list of strings - Will check if the value provided is one of the strings in the list. For example, `collectgarbage` only takes one of a few exact string arguments--doing `collectgarbage("count")` will work, but `collectgarbage("whoops")` won't.
- `{ "display": string }` - Used when no constant could possibly be correct. If a constant is used, selene will tell the user that an argument of the type (display) is required. For an example, the Roblox method `Color3.toHSV` expects a `Color3` object--no constant inside it could be correct, so this is defined as:

```toml
[[Color3.toHSV.args]]
type = { display = "Color3" }
```

## Properties
Example:

```toml
[_VERSION]
property = true
```

Specifies that a property exists. For example, `_VERSION` is available as a global and doesn't have any fields of its own, so it is just defined as a property.

The same goes for `_G`, which is defined as:

```toml
[_G]
property = true
writable = "new-fields"
```

`writable` is an optional field that tells selene how the property can be mutated and used:

- `"new-fields"` - New fields can be added and set, but variable itself cannot be redefined. In the case of _G, it means that `_G = "foo"` is linted against.
- `"overridden"` - New fields can't be added, but entire variable can be overridden. In the case of Roblox's `Instance.Name`, it means we can do `Instance.Name = "Hello"`, but not `Instance.Name.Call()`.
- `"full"` - New fields can be added and entire variable can be overridden.

If `writable` is not specified, selene will assume it can neither have new fields associated with it nor can be overridden.

## Struct
Example:
```toml
[game]
struct = "DataModel"
```

Specifies that the field is an instance of a [struct](#structs). The value is the name of the struct.

## Table
Example:
```toml
[math.huge]
property = true

[math.pi]
property = true
```

A field is understood as a table if it has fields of its own. Notice that `[math]` is not defined anywhere, but its fields are. Fields are of the same type as globals.

## Removed
Example:
```toml
[getfenv]
removed = true
```

Used when your standard library is based off another, and your library removes something from the original.

# Structs

Structs are used in places such as Roblox Instances. Every Instance in Roblox, for example, declares a `:GetChildren()` method. We don't want to have to define this everywhere an Instance is declared globally, so instead we just define it once in a struct.

Structs are defined as fields of `[selene.structs]`. Any fields they have will be used for instances of that struct. For example, the Roblox standard library has the struct:

```toml
[selene.structs.Event.Connect]
method = true

[[selene.structs.Event.Connect.args]]
type = "function"
```

From there, it can define:

```toml
[workspace.Changed]
struct = "Event"
```

...and selene will know that `workspace.Changed:Connect(callback)` is valid, but `workspace.Changed:RandomNameHere()` is not.

# Wildcards
Fields can specify requirements if a field is referenced that is not explicitly named. For example, in Roblox, instances can have arbitrary fields of other instances (`workspace.Baseplate` indexes an instance named Baseplate inside `workspace`, but `Baseplate` is nowhere in the Roblox API).

We can specify this behavior by using the special `"*"` field.

```toml
[workspace."*"]
struct = "Instance"
```

This will tell selene "any field accessed from `workspace` that doesn't exist must be an Instance [struct](#structs)".

Wildcards can even be used in succession. For example, consider the following:

```toml
[script.Name]
property = true
writable = "overridden"

[script."*"."*"]
property = true
writable = "full"
```

Ignoring the wildcard, so far this means:

- `script.Name = "Hello"` *will* work.
- `script = nil` *will not* work, because the writability of `script` is not specified.
- `script.Name.UhOh` *will not* work, because `script.Name` does not have fields.

However, with the wildcard, this adds extra meaning:

- `script.Foo = 3` *will not* work, because the writability of `script.*` is not specified.
- `script.Foo.Bar = 3` *will* work, because `script.*.*` has full writability.
- `script.Foo.Bar.Baz = 3` *will* work for the same reason as above.
