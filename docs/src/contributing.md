# Contributing
selene is written in Rust, so knowledge of the ecosystem is expected.

selene uses [Full Moon](https://github.com/Kampfkarren/full-moon) to parse the Lua code losslessly, meaning whitespace and comments are preserved. You can read the full documentation for Full Moon on its [docs.rs](https://docs.rs/full_moon/latest/full_moon/) page.

TODO: Upload selene-lib on crates.io and link the docs.rs page for that as well as throughout the rest of this article.

## Writing a lint
In selene, lints are created in isolated modules. To start, create a file in `selene-lib/src/rules` with the name of your lint. In this example, we're going to call the lint `cool_lint.rs`.

Let's now understand what a lint consists of. selene takes lints in the form of structs that implement the `Rule` trait. The `Rule` trait expects:

- A `Config` associated type that defines what the configuration format is expected to be. Whatever you pass must be [deserializable](https://serde.rs/).
- An `Error` associated type that implements [`std::error::Error`](https://doc.rust-lang.org/std/error/trait.Error.html). This is used if configurations can be invalid (such as a parameter only being a number within a range). Most of the time, configurations cannot be invalid (other than deserializing errors, which are handled by selene), and so you can set this to [`std::convert::Infallible`](https://doc.rust-lang.org/std/convert/enum.Infallible.html).
- A `new` function with the signature `fn new(config: Self::Config) -> Result<Self, Self::Error>`. With the selene CLI, this is called once.
- A `pass` function with the signature `fn pass(&self, ast: &full_moon::ast::Ast, context: &Context) -> Vec<Diagnostic>`. The `ast` argument is the Full Moon representation of the code, and the context provides optional additional information, such as the standard library being used. Any `Diagnostic` structs returned here are displayed to the user.
- A `severity` function with the signature `fn severity(&self) -> Severity`. Returns either `Severity::Error` or `Severity::Warning`. Use `Error` if the code is positively impossible to be correct. The `&self` is only provided due to limitations of Rust--the function should be completely constant and pure.
- A `rule_type` function with the signature `fn rule_type(&self) -> RuleType`. Returns either `Complexity`, `Correctness`, `Performance`, or `Style`. So far not used for anything. Has the same gotcha as `severity` in relation to `&self`.

For our purposes, we're going to write:

```rs
use super::*;
use std::convert::Infallible;

struct CoolLint;

impl Rule for CoolLint {
    type Config = ();
    type Error = Infallible;

    fn new(_: Self::Config) -> Result<Self, Self::Error> {
        Ok(CoolLint)
    }

    fn pass(&self, ast: &Ast, _: &Context) -> Vec<Diagnostic> {
        unimplemented!()
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn rule_type(&self) -> RuleType {
        RuleType::Style
    }
}
```

The implementation of `pass` is completely up to you, but there are a few common patterns.

- Creating a visitor over the ast provided and creating diagnostics based off of that. See [`divide_by_zero`](https://github.com/Kampfkarren/selene/blob/master/selene-lib/src/rules/divide_by_zero.rs) and [`suspicious_reverse_loop`](https://github.com/Kampfkarren/selene/blob/master/selene-lib/src/rules/suspicious_reverse_loop.rs) for straight forward examples.
- Using the `ScopeManager` struct to lint based off of usage of variables and references. See [`shadowing`](https://github.com/Kampfkarren/selene/blob/master/selene-lib/src/rules/shadowing.rs) and [`global_usage`](https://github.com/Kampfkarren/selene/blob/master/selene-lib/src/rules/global_usage.rs).

### Getting selene to recognize the new lint

Now that we have our lint, we have to make sure selene actually knows to use it. There are two places you need to update.

In selene-lib/src/lib.rs, search for `use_rules!`. You will see something such as:

```rs
use_rules! {
    almost_swapped: rules::almost_swapped::AlmostSwappedLint,
    divide_by_zero: rules::divide_by_zero::DivideByZeroLint,
    empty_if: rules::empty_if::EmptyIfLint,
    ...
}
```

Put your lint in this list (alphabetical order) as the following format:

```
lint_name: rules::module_name::LintObject,
```

For us, this would be:

```
cool_lint: rules::cool_lint::CoolLint,
```

Next, in `selene-lib/src/rules.rs`, search for `pub mod`, and you will see:

```rs
pub mod almost_swapped;
pub mod divide_by_zero;
pub mod empty_if;
...
```

Put your module name in this list, also in alphabetical order.

```rs
pub mod almost_swapped;
pub mod cool_lint;
pub mod divide_by_zero;
pub mod empty_if;
...
```

And we're done! You should be able to `cargo build --bin selene` and be able to use your new lint.

### Writing tests
The selene codebase uses tests extensively for lints. It means we never have to actually build the CLI tool in order to test, and we can make sure we don't have any regressions. **Testing is required if you want to submit your lint to the selene codebase.**

To write a simple test, create a folder in `selene-lib/tests` with the name of your lint. Then, create as many `.lua` files as you want to test. These should contain both cases that do and do not lint. For our cases, we're going to assume our test is called `cool_lint.lua`.

Then, in your lint module, add at the bottom:

```rs
#[cfg(test)]
mod tests {
    use super::{super::test_util::test_lint, *};

    #[test]
    fn test_cool_lint() {
        test_lint(
            CoolLint::new(()).unwrap(),
            "cool_lint",
            "cool_lint",
        );
    }
}
```

Let's discuss what this code means, assuming you're familiar with [the way tests are written and performed in Rust](https://doc.rust-lang.org/book/ch11-00-testing.html).

The `test_lint` function is the easiest way to test that a lint works. It'll search the source code we made before, run selene on it (only your lint), and check its results with the existing `[filename].stderr` file, or create one if it does not yet exist.

The first argument is the lint object to use. `CoolLint::new(())` just means "create `CoolLint` with a configuration of `()`". If your lint specifies a configuration, this will instead be something such as `CoolLintConfig::default()` or whatever you're specifically testing.

The `.unwrap()` is just because `CoolLint::new` returns a `Result`. If you want to test configuration errors, you can avoid `test_lint` altogether and just test `CoolLint::new(...).is_err()` directly.

The first `"cool_lint"` is the name of the folder we created. The second `"cool_lint"` is the name of the *Lua file* we created.

Now, just run `cargo test`, and a `.stderr` file will be automatically generated! You can manipulate it however you see fit as well as modifying your rule, and so long as the file is there, selene will make sure that its accurate.

Optionally, you can add a `.std.toml` with the same name as the test next to the lua file, where you can specify a custom [standard library](./usage/std.html) to use. If you do not, the Lua 5.1 standard library will be used.

### Documenting it

This step is only if you are contributing to the selene codebase, and not just writing personal lints (though I'm sure your other programmers would love if you did this).

To document a new lint, edit `docs/src/SUMMARY.md`, and add your lint to the table of contents along the rest. As with everything else, make sure it's in alphabetical order.

Then, edit the markdown file it creates (if you have `mdbook serve` on, it'll create it for you), and write it in this format:

````
# rule_name
## What it does
Explain what your lint does, simply.

## Why this is bad
Explain why a user would want to lint this.

## Configuration
Specify any configuration if it exists.

## Example
```lua
-- Bad code here
```

...should be written as...

```lua
-- Good code here
```

## Remarks
If there's anything else a user should know when using this lint, write it here.
````

This isn't a strict format, and you can mess with it as appropriate. For example, `standard_library` does not have a "Why this is bad" section as not only is it a very encompassing rule, but it should be fairly obvious. Many rules don't specify a "...should be written as..." as it is either something with various potential fixes (such as [`global_usage`](./lints/global_usage.md)) or because the "good code" is just removing parts entirely (such as [`unbalanced_assignments`](./lints/unbalanced_assignments.md)).
