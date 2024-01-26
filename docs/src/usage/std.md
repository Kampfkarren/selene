# Standard Library Format
selene provides a robust standard library format to allow for use with environments other than vanilla Lua. Standard libraries are defined in the form of [YAML](https://en.wikipedia.org/wiki/YAML) files.

## Examples
For examples of the standard library format, see:
- [`lua51.yml`](https://github.com/Kampfkarren/selene/blob/main/selene-lib/default_std/lua51.yml) - The default standard library for Lua 5.1
- [`lua52.yml`](https://github.com/Kampfkarren/selene/blob/main/selene-lib/default_std/lua52.yml) - A standard library for Lua 5.2's additions and removals. Reference this if your standard library is based off another (it most likely is).
- [`roblox.yml`](https://gist.github.com/Kampfkarren/dff2dc17cc30d68a48510da58fff2381) - A standard library for Roblox that incorporates all the advanced features of the format. If you are a Roblox developer, don't use this as anything other than reference--an up to date version of this library is automatically generated.

## base

Used for specifying what standard library to be based off of. This supports both builtin libraries (lua51, lua52, lua53, roblox), as well as any standard libraries that can be found in the current directory.

```yaml
--- # This begins a YAML file
base: lua51 # We will be extending off of Lua 5.1.
```

## globals
This is where the magic happens. The `globals` field is a dictionary where the keys are the globals you want to define. The value you give tells selene what the value can be, do, and provide.

If your standard library is based off another, overriding something defined there will use your implementation over the original.

### Any
```yaml
---
globals:
  foo:
    any: true
```

This specifies that the field can be used in any possible way, meaning that `foo.x`, `foo:y()`, etc will all validate.

### Functions
```yaml
---
globals:
  tonumber:
    args:
      - type: any
      - type: number
        required: false
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

#### "required"
- `true` - The default, this argument is required.
- `false` - This argument is optional.
- A string - This argument is required, and not using it will give this as the reason why.

#### "observes"
This field is used for allowing smarter introspection of how the argument given is used.

- "read-write" - The default. This argument is potentially both written to and read from.
- "read" - This argument is only read from. Currently unused.
- "write" - This argument is only written to. Used by `unused_variable` to assist in detecting a variable only being written to, even if passed into a function.

Example:
```yml
 table.insert:
  args:
    - type: table
      observes: write # This way, `table.insert(x, 1)` doesn't count as a read to `x`
    - type: any
    - required: false
      type: any
```

#### "must_use"
This field is used for checking if the return value of a function is used.

- `false` - The default. The return value of this function does not need to be used.
- `true` - The return value of this function must be used.

Example:
```yml
table.find:
  args:
    - type: table
      observes: read
      required: true
    - type: any
      required: true
  must_use: true  # The return value of this function must be used
```

#### Argument types
- `"any"` - Allows any value.
- `"bool"`, `"function"`, `"nil"`, `"number"`, `"string"`, `"table"` - Expects a value of the respective type.
- `"..."` - Allows any number of variables after this one. If `required` is true (it is by default), then this will lint if no additional arguments are given. It is incorrect to have this in the middle.
- Constant list of strings - Will check if the value provided is one of the strings in the list. For example, `collectgarbage` only takes one of a few exact string arguments--doing `collectgarbage("count")` will work, but `collectgarbage("whoops")` won't.
- `{ "display": string }` - Used when no constant could possibly be correct. If a constant is used, selene will tell the user that an argument of the type (display) is required. For an example, the Roblox method `Color3.toHSV` expects a `Color3` object--no constant inside it could be correct, so this is defined as:

```yaml
---
globals:
  Color3.toHSV:
    args:
      - type:
          display: Color3
```

### Properties
```yaml
---
globals:
  _VERSION:
    property: read-only
```

Specifies that a property exists. For example, `_VERSION` is available as a global and doesn't have any fields of its own, so it is just defined as a property.

The same goes for `_G`, which is defined as:
```yaml
_G:
  property: new-fields
```

The value of property tells selene how it can be mutated and used:

- `"read-only"` - New fields cannot be added or set, and the variable itself cannot be redefined.
- `"new-fields"` - New fields can be added and set, but variable itself cannot be redefined. In the case of _G, it means that `_G = "foo"` is linted against.
- `"override-fields"` - New fields can't be added, but entire variable can be overridden. In the case of Roblox's `Instance.Name`, it means we can do `Instance.Name = "Hello"`, but not `Instance.Name.Call()`.
- `"full-write"` - New fields can be added and entire variable can be overridden.

### Struct
```yaml
---
globals:
  game:
    struct: DataModel
```

Specifies that the field is an instance of a [struct](#structs). The value is the name of the struct.

### Tables
```yaml
---
globals:
  math.huge:
    property: read-only
  math.pi:
    property: read-only
```

A field is understood as a table if it has fields of its own. Notice that `math` is not defined anywhere, but its fields are. This will create an implicit `math` with the property writability of `read-only`.

### Deprecated
Any field can have a deprecation notice added to it, which will then be read by [the deprecated lint](../lints/deprecated.md).

```yaml
---
globals:
  table.getn:
    args:
      - type: table
      - type: number
    deprecated:
      message: "`table.getn` has been superseded by #."
      replace:
        - "#%1"
```

The deprecated field consists of two subfields.

`message` is required, and is a human readable explanation of what the deprecation is, and potentially why.

`replace` is an optional array of replacements. The most relevant replacement is suggested to the user. If used with a function, then every parameter of the function will be provided.

For instance, since `table.getn`'s top replacement is `#%1`:
- `table.getn(x)` will suggest `#x`
- `table.getn()` will not suggest anything, as there is no relevant suggestion

You can also use `%...` to list every argument, separated by commas.

The following:
```yaml
---
globals:
  call:
    deprecated:
      message: "call will be removed in the next version"
      replace:
        - "newcall(%...)"
    args:
      - type: "..."
        required: false
```

...will suggest `newcall(1, 2, 3)` for `call(1, 2, 3)`, and `newcall()` for `call()`.

You can also use `%%` to write a raw `%`.

### Removed
```yaml
---
globals:
  getfenv:
    removed: true
```

Used when your standard library is [based off](#base) another, and your library removes something from the original.

## Structs
Structs are used in places such as Roblox Instances. Every Instance in Roblox, for example, declares a `:GetChildren()` method. We don't want to have to define this everywhere an Instance is declared globally, so instead we just define it once in a struct.

Structs are defined as fields of `structs`. Any fields they have will be used for instances of that struct. For example, the Roblox standard library has the struct:

```yaml
---
structs:
  Event:
    Connect:
      method: true
      args:
        - type: function
```

From there, it can define:

```yaml
globals:
  workspace.Changed:
    struct: Event
```

...and selene will know that `workspace.Changed:Connect(callback)` is valid, but `workspace.Changed:RandomNameHere()` is not.

## Wildcards
Fields can specify requirements if a field is referenced that is not explicitly named. For example, in Roblox, instances can have arbitrary fields of other instances (`workspace.Baseplate` indexes an instance named Baseplate inside `workspace`, but `Baseplate` is nowhere in the Roblox API).

We can specify this behavior by using the special `"*"` field.

```yaml
workspace.*:
  struct: Instance
```

This will tell selene "any field accessed from `workspace` that doesn't exist must be an Instance [struct](#structs)".

Wildcards can even be used in succession. For example, consider the following:

```yaml
script.Name:
  property: override-fields

script.*.*:
  property: full-write
```

Ignoring the wildcard, so far this means:

- `script.Name = "Hello"` *will* work.
- `script = nil` *will not* work, because the writability of `script` is not specified.
- `script.Name.UhOh` *will not* work, because `script.Name` does not have fields.

However, with the wildcard, this adds extra meaning:

- `script.Foo = 3` *will not* work, because the writability of `script.*` is not specified.
- `script.Foo.Bar = 3` *will* work, because `script.*.*` has full writability.
- `script.Foo.Bar.Baz = 3` *will* work for the same reason as above.

## Internal properties

There are some properties that exist in standard library YAMLs that exist specifically for internal purposes. This is merely a reference, but these are not guaranteed to be stable.

### name

This specifies the name of the standard library. This is used internally for cases such as only giving Roblox lints if the standard library is named `"roblox"`.

### last_updated

A timestamp of when the standard library was last updated. This is used by the Roblox standard library generator to update when it gets too old.

### last_selene_version

A timestamp of the last selene version that generated this standard library. This is used by the Roblox standard library generator to update when it gets too old.

### roblox_classes

A map of every Roblox class and their properties, for [roblox_incorrect_roact_usage](../lints/roblox_incorrect_roact_usage.md).
