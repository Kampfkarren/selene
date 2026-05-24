# Configuration
selene is meant to be easily configurable. You can specify configurations for the entire project as well as for individual lints.

Configuration files apply to the directory (and subdirectories) in which they are placed. So, to control how selene behaves when it lints scripts in, say, `~/myScripts` and below, place the configuration file within (the top-level of) `~/myScripts`. Each configuration file is to be called **selene.toml**. As the file suffix suggests, configuration files use the [Tom's Obvious, Minimal Language (TOML)](https://github.com/toml-lang/toml) format. It is recommended you quickly brush up on the syntax, though it is very easy.

## Changing the severity of lints
You can change the severity of lints by entering the following into selene.toml:

```toml
[lints]
lint_1 = "severity"
lint_2 = "severity"
...
```

where "severity" is one of the following:

- `"allow"` - don't check for this lint;
- `"warn"` - produce a warning when this lint is transgressed;
- `"deny"` - produce an error message when this lint is transgressed.

`deny` and `warn` are near-identical in effect. The only difference are: errors are printed in red, and warnings in orange; errors increment the error counter, warnings, the warning counter.

## Configuring specific lints
You can configure specific lints by entering the following into selene.toml:

```toml
[config]
lint1 = ...
lint2 = ...
...
```

Where the value is whatever the special configuration of that lint is. You can learn these on the lints specific page in the [list of lints](../lints/index.md). For example, if we wanted to allow empty if branches if the contents contain comments, then we would write:

```toml
[config]
empty_if = { comments_count = true }
```

## Setting the standard library
Many lints use standard libraries for either verifying their correct usage or for knowing that variables exist where they otherwise wouldn't.

By default, selene uses Lua 5.1, though if we wanted to use the Lua 5.2 standard library, we would write:

```toml
std = "lua52"
```

...at the top of selene.toml. You can learn more about the standard library format on the [standard library guide](./std.md). The standard library given can either be one of the builtin ones (currently only `lua51` and `lua52`) or the filename of a standard library file in this format. For example, if we had a file named `special.toml`, we would write:

```toml
std = "special"
```

### Chaining the standard library

We can chain together multiple standard libraries by simply using a plus sign (`+`) in between the names.

For example, if we had `game.toml` and `engine.toml` standard libraries, we could chain them together like so:

```toml
std = "game+engine"
```

### Excluding files from being linted
It is possible to exclude files from being linted using the exclude option:

```toml
exclude = ["external/*", "*.spec.lua"]
```
