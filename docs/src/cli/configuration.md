# Configuration
selene is meant to be easily configurable. You can specify configurations for the entire project as well as for individual lints.

Configuration files are placed in the directory you are running selene in and are named **selene.toml**. As the name suggests, the configurations use the [Tom's Obvious, Minimal Language (TOML)](https://github.com/toml-lang/toml) format. It is recommended you quickly brush up on the syntax, though it is very easy.

## Changing the severity of lints
You can change the severity of lints by entering the following into selene.toml:

```toml
[config]
lint_1 = "severity"
lint_2 = "severity"
...
```

Where "severity" is one of the following:

- `"allow"` - Don't check for this lint
- `"warn"` - Warn for this lint
- `"deny"` - Error for this lint

Note that "deny" and "warn" are effectively the same, only warn will give orange text while error gives red text, and they both have different counters.

## Configuring specific rules
You can configure specific rules by entering the following into selene.toml:

```toml
[rules]
rule1 = ...
rule2 = ...
...
```

Where the value is whatever the special configuration of that rule is. You can learn these on the lints specific page in the [list of lints](../lints/index.md). For example, if we wanted to allow empty if branches if the contents contain comments, then we would write:

```toml
[rules]
empty_if = { comments_count = false }
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
