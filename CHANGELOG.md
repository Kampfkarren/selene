# Changelog
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased](https://github.com/Kampfkarren/selene/compare/0.27.1...HEAD)
### Added
- Added `Path2DControlPoint.new` to the Roblox standard library
- [Adds `lua_versions` to standard library definitions](https://kampfkarren.github.io/selene/usage/std.html#lua_versions). Specifying this will only allow the syntax used by those languages. The default standard libraries now specify these, meaning that invalid syntax for that language will no longer be supported.
- Added missing third parameter to `PathWaypoint.new` in the Roblox standard library
- Added `vector` library to Luau standard library
- Added `math.map` to the Luau standard library

### Changed
- Upgrades to [full-moon 1.0.0](https://github.com/Kampfkarren/full-moon/blob/main/CHANGELOG.md#100---2024-10-08), which should provide faster parse speeds, support for multiple parsing errors at the same time, and support for some new Luau syntax.

## [0.27.1](https://github.com/Kampfkarren/selene/releases/tag/0.27.1) - 2024-04-28
### Fixed
- Fixed `Instance.new`'s second parameter being incorrectly marked as required.

## [0.27.0](https://github.com/Kampfkarren/selene/releases/tag/0.27.0) - 2024-04-28
### Added
- Added `CFrame.lookAlong` to the Roblox standard library
- Added `deprecated` config field to standard library function parameters

### Changed
- Updated the warning message for the `mixed_table` lint to include why mixed tables should be avoided
- Properly deprecated `Instance.new`'s second argument in the Roblox standard library

## [0.26.1](https://github.com/Kampfkarren/selene/releases/tag/0.26.1) - 2023-11-11
### Fixed
- Fixed `UDim2.new()` firing the [`roblox_suspicious_udim2_new` lint](https://kampfkarren.github.io/selene/lints/roblox_suspicious_udim2_new.html).

## [0.26.0](https://github.com/Kampfkarren/selene/releases/tag/0.26.0) - 2023-11-11
### Added
- Added `table.move` and `math.tointeger` to Lua 5.3 standard library
- Added `bit32.*` functions to Lua 5.2 standard library
- Added `table.pack`, `rawlen` and `package.config` to Lua 5.2 standard library
- Added new [`empty_loop` lint](https://kampfkarren.github.io/selene/lints/empty_loop.html), which will check for empty loop blocks.
- Added new [`roblox_suspicious_udim2_new` lint](https://kampfkarren.github.io/selene/lints/roblox_suspicious_udim2_new.html), which will warn when you pass in too few number of arguments to `UDim2.new`.
- `roblox_incorrect_roact_usage` now lints for illegal `Name` property
- Added `ignore_pattern` config to `global_usage`, which will ignore any global variables with names that match the pattern
- `roblox_incorrect_roact_usage` now checks for incorrect Roact17's `createElement` usage on variables named `React`. For Roact17 only, `key`, `children`, and `ref` are valid properties to Roblox instances.
- Excludes are now respected for single files.
- Added `no-exclude` cli flag to disable excludes.
- When given in standard library format, additional information now shows up in `incorrect_standard_library_use` missing required parameter errors.
- Added new [`mixed_table` lint](https://kampfkarren.github.io/selene/lints/mixed_table.html), which will warn against mixed tables.
- Added `bit32.byteswap` to Luau standard library
- Added `buffer` library to Luau standard library
- Added `SharedTable` to Roblox standard library

### Changed
- Updated internal parser, which includes floor division (`//`), more correct parsing of string interpolation with double braces, and better parsing of `\z` escapes.

### Fixed
- `string.pack` and `string.unpack` now have proper function signatures in the Lua 5.3 standard library.
- Moved `math.log` second argument addition from Lua 5.3 std lib to 5.2 std lib
- `undefined_variable` now correctly errors when defining multiple methods in undefined tables
- Corrected `os.exit` definition in Lua 5.2 standard library
- Fixed `manual_table_clone` incorrectly warning when loop and table are defined at different depths

## [0.25.0](https://github.com/Kampfkarren/selene/releases/tag/0.25.0) - 2023-03-12
### Added
- Added `CFrame.fromEulerAngles` to the Roblox standard library.
- Added `validate-config` command, which will report any errors in your configuration.
- Added `capabilities` command, which will report the feature set of the selene installation. This is useful for consumers like the VS Code extension.

### Changed
- Unknown keys in configuration files are accepted less often now.
- Updated internal parser, supporting Chinese characters better.

### Fixed
- "Legacy" Roblox enums (such as Enum.RaycastFilterType.Whitelist/Blacklist) are now automatically created and marked as deprecated in generated standard libraries.
- Fixed a bug where `manual_table_clone` would incorrectly lint code in loops. ([#479](https://github.com/Kampfkarren/selene/issues/479))

## [0.24.0](https://github.com/Kampfkarren/selene/releases/tag/0.24.0) - 2023-01-10
### Added
- Added new [`manual_table_clone` lint](https://kampfkarren.github.io/selene/lints/manual_table_clone.html), which will catch manual re-implementations of `table.clone` in Luau.
- Added `filename` field to diagnostic message labels in JSON output, indicating for which file the message was generated (#453)

### Changed
- Improved the error message for using a standard library that can be detected as outdated.
- Updated internal parser, giving support for string interpolation for Luau and fixing some parsing bugs.

### Fixed
- Fixed "library" being typo'd as "libary" in the error when finding a usage.

## [0.23.1](https://github.com/Kampfkarren/selene/releases/tag/0.23.1) - 2022-12-06
### Fixed
- Fixed event warnings not being possible to filter out with `roblox_incorrect_roact_usage`.

## [0.23.0](https://github.com/Kampfkarren/selene/releases/tag/0.23.0) - 2022-12-06
### Added
- Added `--display-style=json2`, which gives the same outputs as `--display-style=json`, but with an extra `type` field so that it can support more than diagnostics. Extensions should move over to `--display-style=json2` as more becomes available for it, but take care to check for `type`. Currently the only possible value is "Diagnostic".
- Added `rawlen` to the Luau standard library.
- Added `Font.fromEnum`, `Font.fromName`, and `Font.fromId` to the Roblox standard library.
- Added the missing `table.foreachi` function to Lua 5.1 standard library as deprecated.

### Fixed
- `warn` in the Roblox standard library now properly works with all data types instead of only strings.

## [0.22.0](https://github.com/Kampfkarren/selene/releases/tag/0.22.0) - 2022-10-15
### Added
- Added `--allow-warnings` option to have selene pass when only warnings occur.
- Added the ability to allow specific patterns in the `deprecated` lint.
- Added exclude option to selene.toml for excluding files from lints
- Adds support for `.yaml` extensions to be used for standard libraries alongside `.yml`.
- Normalized "lint" terminology over "rule" throughout codebase. "rules" in `selene.toml` should now be "lints", but "rules" will still be supported for backwards compatibility.

### Changed
- Updated internal parser, giving substantial parsing speed increases.

## [0.21.1](https://github.com/Kampfkarren/selene/releases/tag/0.21.0) - 2022-09-19
### Fixed
- Fixed not being able to use projects without selene.toml.

## [0.21.0](https://github.com/Kampfkarren/selene/releases/tag/0.21.0) - 2022-09-17
### Added
- `undefined_variable` now properly catches `global` as undefined in `function global.name()`.
- Added the "luau" builtin library.
- `unused_variable` and `incorrect_standard_library_use` will now suggest configuring a standard library if one is detected.
- Added `constant_table_comparison` check to catch `x == {}`, which will always fail.
- Added `high_cyclomatic_complexity` check to catch overly complex functions that are hard to test, and harder to reason about. This lint is disabled by default.
- Added `Font.new` to the Roblox standard library.
- `roblox_incorrect_roact_usage` now lints for invalid events.

### Changed
- Match `.luau` filename extension by default.
- Allow `--pattern` to be passed multiple times.
- `roblox_incorrect_roact_usage` now uses the generated standard library to know what classes and properties exist, meaning a selene update is no longer necessary to update.
- Roblox standard libraries are now guaranteed to regenerate when the previously generated standard library is on a different version.

### Fixed
- Fixed `unused_variable` incorrectly tagging `function global.name()` when `global` is defined in the standard library.
- Fixed `unscoped_variables` incorrectly tagging `function global.name()` as creating an unscoped variable for `global`.
- Fixed `roblox_incorrect_roact_usage` always showing the class name as "Instance". ([#297](https://github.com/Kampfkarren/selene/issues/297))
- `roblox_incorrect_roact_usage` will now find instances of createElement that do not specify properties.
- Fixed issues where `roblox_incorrect_color3_new_bounds` would sometimes fail to run.

## [0.20.0](https://github.com/Kampfkarren/selene/releases/tag/0.20.0) - 2022-07-21
### Added
- Added [`utf8` globals](https://q-syshelp.qsc.com/Content/Control_Scripting/Lua_5.3_Reference_Manual/Standard_Libraries/4_-_Basic_UTF-8_Support.htm) to the builtin `lua53` standard library.
- Added Roblox datatype constructors `CatalogSearchParams.new`, `FloatCurveKey.new`, and `RotationCurveKey.new`.

### Changed
- Errors for generating Roblox API dumps are now more detailed.

### Fixed
- Fixed newer versions of the Roblox API dump failing to create standard libraries for.
- Fixed reporting an error when generating standard libraries panicking.

## [0.19.1](https://github.com/Kampfkarren/selene/releases/tag/0.19.1) - 2022-06-22
### Fixed
- Fixed releases coming with Tracy.

## [0.19.0](https://github.com/Kampfkarren/selene/releases/tag/0.19.0) - 2022-06-22
### Added
- `table.insert(x)` no longer counts as a read to `x`, which allows selene to alert you that you are only assigning to it.
  - This is done through a new standard library field for arguments called `observes`. This takes 3 values: "read-write" (the default), signifying a potential read and write, "read", signifying only a read, and "write", signifying only a write. Only "write" has meaning at this time.
- Added new `must_use` lint, which will warn you when you are not using the return value of a function that performs no other behavior.
  - This is done through a new standard library field for functions called `must_use`. Set it to `true` to gain this functionality.

### Fixed
- Fixed a bunch of performance failures, lowering some benchmarks from 3 seconds to 200ms.

## [0.18.2](https://github.com/Kampfkarren/selene/releases/tag/0.18.2) - 2022-06-10
### Fixed
- Fixed `Enum.NAME.Value` failing in newly generated standard libraries.

## [0.18.1](https://github.com/Kampfkarren/selene/releases/tag/0.18.1) - 2022-06-07
### Changed
- [Updated internal parser](https://github.com/Kampfkarren/full-moon/blob/main/CHANGELOG.md#0151---2022-02-17), bringing bug fixes to type information with generic packs.

## [0.18.0](https://github.com/Kampfkarren/selene/releases/tag/0.18.0) - 2022-06-07
### Added
- Added [new YAML based standard library format](https://kampfkarren.github.io/selene/usage/std.html). The old TOML format is now deprecated and will not have any new functionality added to it, but will be preserved for the forseeable future.
  - You can upgrade old TOML standard libraries by running `selene upgrade-std library.toml`, which will create a new .yml file of the same name in the new format.
  - This only affects **standard library files**. `selene.toml` has not changed.
- Added new [`deprecated` lint](https://kampfkarren.github.io/selene/lints/deprecated.html), which can be configured [by standard libraries](https://kampfkarren.github.io/selene/usage/std.html#deprecated).
- Added `debug.resetmemorycategory` to the Roblox standard library.
- Added `debug.setmemorycategory` to the Roblox standard library.
- Added `--no-summary` option to suppress summary information.

### Changed
- Roblox standard library files are now no longer generated in the project directory, and will be updated automatically every 6 hours. You can update it manually with `selene update-roblox-std`.
  - As per the deprecation of TOML standard libraries, you should delete your `roblox.toml` if you have one.
  - It is possible to pin a standard library in the same way `roblox.toml` was if you are in an environment where you do not want automatic updates, such as one where you want to limit selene's internet usage. Learn more [on the Roblox Guide documentation page](https://kampfkarren.github.io/selene/roblox.html).

### Removed
- With the introduction of the new `deprecated` lint, the `--deprecated` field has been removed from `generate-roblox-std`, and is now implied.

## [0.17.0](https://github.com/Kampfkarren/selene/releases/tag/0.17.0) - 2022-04-10
### Added
- Added `start_line`, `start_column`, `end_line`, and `end_column` to JSON diagnostic output.
- Added `Color3.fromHex` to the Roblox standard library.
- Added `table.clone` to the Roblox standard library.
- Added `coroutine.close` to the Roblox standard library.
- Added `task.cancel` to the Roblox standard library.

## [0.16.0](https://github.com/Kampfkarren/selene/releases/tag/0.16.0) - 2022-01-30
### Added
- Added support for parsing generic type packs, variadic type packs, and explicit type packs in generic arguments for a type under the `roblox` feature flag (`type X<S...> = Y<(string, number), ...string, S...>`)
- Added support for string and boolean singleton types under the `roblox` feature flag (`type Element = { ["$$typeof"]: number, errorCaught: true, which: "Query" | "Mutation" | "Subscription" }`
- Added support for default types in a generic type declaration under the `roblox` feature flag (`type Foo<X = string> = X`)
- Added `table.freeze`, `table.isfrozen`, `bit32.countlz`, `bit32.countrz` to the Roblox standard library.
- Added `Vector2.zero`, `Vector2.one`, `Vector2.xAxis`, `Vector2.yAxis` to the Roblox standard library.
- Added `Vector3.zero`, `Vector3.one`, `Vector3.xAxis`, `Vector3.yAxis`, `Vector3.zAxis` to the Roblox standard library.
- Added `CFrame.identity` to the Roblox standard library.
- Added `gcinfo` to the Roblox standard library.

### Fixed
- Fixed a bug where empty else blocks were not properly closing their scope, meaning that they could confuse the shadowing lint. [(#116)](https://github.com/Kampfkarren/selene/issues/116)

## [0.15.0](https://github.com/Kampfkarren/selene/releases/tag/0.15.0) - 2021-11-05
### Added
- Added `OverlapParams` to the Roblox standard library.
- Added `Enum:GetEnums()` to the Roblox standard library. [(#312)](https://github.com/Kampfkarren/selene/issues/312)
- `roblox_incorrect_color3_new_bounds` now checks for if the given number is negative. [(#83)](https://github.com/Kampfkarren/selene/issues/83)

### Fixed
- Fixed standard library error when missing files. [(#272)](https://github.com/Kampfkarren/selene/issues/272)
- Fixed display style option triggering `ArgumentConflict` when using quiet option. [(#288)](https://github.com/Kampfkarren/selene/issues/288)
- `bad_string_escape` now correctly handles escapes of the shape `\1a` (one or two numbers followed by a hex digit). [(#292)](https://github.com/Kampfkarren/selene/issues/292)
- Fixed Roblox types not counting towards usage. [(#270)](https://github.com/Kampfkarren/selene/issues/270)
- Fixed incorrect number of paremeters for `bit32.replace`

### Changed
- `duplicate_keys` now has a error severity. [(#262)](https://github.com/Kampfkarren/selene/issues/262)
- Arguments of `collectgarbage` now considered to be optional. [(#287)](https://github.com/Kampfkarren/selene/issues/287)
- Updated internal parser, adding new Luau syntax.

## [0.14.0](https://github.com/Kampfkarren/selene/releases/tag/0.14.0) - 2021-07-07
### Added
- Added `task` library to Roblox standard library.

### Changed
- `mismatched_arg_count` now tries to find the best overlap between multiple definitions, rather than ignoring them entirely. This means that if you have `f(a)` and `f(b, c)` defined, then calling `f(1, 2, 3)` will now lint instead of silently passing, since no definition provided meets it.
- `mismatched_arg_count` now shows all function definitions, rather than the local variable assignment. [(#259)](https://github.com/Kampfkarren/selene/issues/259)

### Fixed
- Updated internal parser, fixing some bugs with Roblox parsing. [See the changelog here](https://github.com/Kampfkarren/full-moon/blob/main/CHANGELOG.md#0131---2021-07-07).

## [0.13.0](https://github.com/Kampfkarren/selene/releases/tag/0.13.0) - 2021-07-01
### Added
- Added `debug.info` to the Roblox standard library. [(#260)](https://github.com/Kampfkarren/selene/issues/260)
- Tokenization errors now form rich diagnostics.

### Changed
- Updated internal parser.
- Optimized linting process to run better with multiple threads.

### Fixed
- Fixed internal selene panics exiting with a zero code. Panics will now exit with status code 1, allowing it to be picked up by CI.
- Fixed variables named `self` not showing as unused if `allow_unused_self` was enabled. The implicit `self` variable being unused will still respect this configuration. [(#215)](https://github.com/Kampfkarren/selene/issues/215)

## [0.12.1](https://github.com/Kampfkarren/selene/releases/tag/0.12.1) - 2021-05-26
### Fixed
- Fixed compile warning about unused variables.

## [0.12.0](https://github.com/Kampfkarren/selene/releases/tag/0.12.1) - 2021-05-26
### Added
- `arg` is now defined in the Lua 5.1 standard library.
- Root level `...` will no longer be declared an undefined variable.
- Using `...` past the first depth will now error with `undefined_variable`, as it is guaranteed improper code.
- You can now combine a function with fields inside standard libraries. This is useful for something like `expect()` and `expect.extend()`.
- Added `mismatched_arg_count` lint which will lint against too many arguments passed to calls for locally defined functions.
- Added `duplicate_keys` lint for checking keys defined more than once inside a table.

### Fixed
- Fixed a bug where some indexes of Roblox structures would fail (such as `workspace.CurrentCamera.ViewportSize.X`)
- Fixed a bug where chaining `roblox` with another standard library would not read the other standard library if `roblox` was being generated.
- Fixed a bug where `0.5 * x` would always resolve to a number.

### Changed
- Updated internal parser. This has shown in practice to catch lints that the previous version did not.

## [0.11.0](https://github.com/Kampfkarren/selene/releases/tag/0.11.0) - 2021-01-04
### Added
- Added support for DateTime in the Roblox standard library.
- Added support for `table.clear` in the Roblox standard library.

## [0.10.1](https://github.com/Kampfkarren/selene/releases/tag/0.10.1) - 2020-12-22
### Fixed
- Fixed regressions related to numeric for loops.

## [0.10.0](https://github.com/Kampfkarren/selene/releases/tag/0.10.0) - 2020-12-21
### Added
- Added inline lint filtering, read [the documentation](https://kampfkarren.github.io/selene/usage/filtering.html) for more information.
- More errors now set the exit code.
- Added support for error({any}) to the Roblox standard library.
- Added initial support for Lua 5.3 in the "lua53" standard library:
    - New function `string.pack`
    - New function `string.unpack`
    - New function `string.packsize`
    - New optional arg for `math.log`

### Changed
- `UDim.new` and `Region3int16.new` no longer require parameters.
- `UDim2.fromOffset` and `UDim2.fromScale` now require you to use `UDim.new` if no parameters are specified.
- Updated full-moon, read [the full-moon changelog](https://github.com/Kampfkarren/full-moon/blob/master/CHANGELOG.md#080---2020-12-21) to learn more.

## [0.9.2](https://github.com/Kampfkarren/selene/releases/tag/0.9.2) - 2020-11-06
### Changed
- Updated full-moon, read [the full-moon changelog](https://github.com/Kampfkarren/full-moon/blob/master/CHANGELOG.md#070---2020-11-06) to learn more.

## [0.9.1](https://github.com/Kampfkarren/selene/releases/tag/0.9.1) - 2020-11-04
### Fixed
- Fixed `--display-style=json` giving an output incompatible with previous tooling.

## [0.9.0](https://github.com/Kampfkarren/selene/releases/tag/0.9.0) ([Notes](https://github.com/Kampfkarren/selene/releases/tag/0.9.1)) - 2020-11-04
### Added
- Arguments that aren't required can now be filled with nil.
- Added support for `math.round` to the Roblox standard library.
- Added support for `CFrame.lookAt` to the Roblox standard library.

### Changed
- setmetatable no longer requires a second argument.
- `allow_unused_self` is now toggled on for `unused_variable` by default.
- Updated local Roblox reflection for more up to date instances.

### Fixed
- Using a function call as the last argument in a function will silence lint for not passing enough parameters. This means, for example, `math.max(unpack(numbers))` will no longer error.
- Using an ellipsis on the right side of unbalanced assignments no longer lints.

## [0.8.0](https://github.com/Kampfkarren/selene/releases/tag/0.8.0) - 2020-08-24
### Added
- Added support for `os.clock`.
- Added `RaycastParams.new`.
- Added support for `string.pack`, `string.packsize`, and `string.unpack` to the Roblox standard library.
- Added lint `compare_nan` to guard against comparing directly to nan (e.g. `x ~= 0/0`).
- Add lint `bad_string_escape` to guard invalid or malformed string escape sequences.

### Fixed
- Fixed `coroutine.yield` only accepting coroutines as a first argument.

## [0.7.0](https://github.com/Kampfkarren/selene/releases/tag/0.7.0) - 2020-06-08
### Added
- Added support for `continue`, compound assignments (`+`), intersectional types, and numbers with underscores under the `roblox` feature flag.

### Fixed
- Fixed several parse errors with numbers.

### Changed
- Removed types from `debug.traceback` arguments in the Lua 5.1 standard library
- Made 4th argument to `CFrame.fromMatrix` optional (#113)
- Made standard library aware that functions and `...` can return multiple values

## [0.6.0](https://github.com/Kampfkarren/selene/releases/tag/0.6.0) - 2020-04-21
### Added
- Added timestamp to generated Roblox standard library
- Added `debug.getlocal`, `math.cosh`, and `string.reverse`
- Added `package` library
- Added `Axes`, `Faces`, `PathWaypoint`,  to the Roblox standard library
- Added `DebuggerManager`, `elapsedTime` to the Roblox standard library

### Fixed
- Corrected arguments for `assert`, `xpcall`, `coroutine.yield`, `debug.getinfo`, `debug.setfenv`, `string.char`, and `string.gsub` in Lua 5.1 standard library
- Corrected arguments for `bit32.band`, `Color3.toHSV`, `Rect.new`, and `UDim2.new` in the Roblox standard library
- `require` now accepts numbers in the Roblox standard library
- Removed `string.dump` from the Roblox standard library
- Fixed a bug where the `almost_swapped` lint would panic if the last line was an assignment [(#93)](https://github.com/Kampfkarren/selene/issues/93)

### Changed
- Changed the argument display type of `io.input` and `io.output` into `file`
- Updated to version 0.5.0 of full-moon, which should result in speedier parsing

## [0.5.3](https://github.com/Kampfkarren/selene/releases/tag/0.5.3) - 2020-01-27
### Added
- Added tuple argument to `xpcall`
- Added `CFrame.fromEulerAnglesYXZ` to Roblox standard library
- Added `ColorSequenceKeypoint` to the Roblox standard library

### Fixed
- Fixed comments with tabs reporting as a parse error.

## [0.5.2](https://github.com/Kampfkarren/selene/releases/tag/0.5.2) - 2020-01-19
### Fixed
- Fixed debug output for the standard library.

## [0.5.1](https://github.com/Kampfkarren/selene/releases/tag/0.5.2) - 2020-01-19
### Added
- Added the `utf8` library to the Roblox standard library
- Added support for Typed Lua when using the Roblox feature flag.

### Changed
- Updated full-moon, which should result in faster parsing.

## [0.5.0](https://github.com/Kampfkarren/selene/releases/tag/0.5.0) - 2019-12-20
### Added
- Added `type_check_inside_call` lint for checking `type(foo == "type")` instead of `type(foo) == "type"`.
- Added `NumberRange` to the Roblox standard library.
- Added `string.split` to the Roblox standard library.
- Added `table.find` to the Roblox standard library.
- Added `table.create` to the Roblox standard library.
- Added `table.move` to the Roblox standard library.
- Added `table.pack` to the Roblox standard library.
- Added `table.unpack` to the Roblox standard library.
- Added `coroutine.yieldable` to the Roblox standard library.
- Added second argument to `math.log` to the Roblox standard library.
- Added `NumberSequenceKeypoint` to the Roblox standard library.

### Fixed
- Fixed ternary expressions resolving as booleans.

## [0.4.3](https://github.com/Kampfkarren/selene/releases/tag/0.4.3) - 2019-11-20
### Added
- Added `display-style` flag to use either rich (default), quiet (equivalent to -q), or JSON.

### Fixed
- Fixed a concatenated result always triggering an error when the standard library function expected a constant string (such as `collectgarbage` or `Instance.new`).
- Fixed parenthese conditions mixed with non-parenthese conditions (such as `(condition) and condition`) tripping the `parenthese_conditions` lint.

## [0.4.2](https://github.com/Kampfkarren/selene/releases/tag/0.4.2) - 2019-11-13
### Fixed
- Fixed Roblox standard library not including structs, and thus failing when using `game`, `script`, etc.

## [0.4.1](https://github.com/Kampfkarren/selene/releases/tag/0.4.1) - 2019-11-13
### Fixed
- Fixed Roblox standard library not including Lua 5.1 globals the first time you ran selene.

## [0.4.0](https://github.com/Kampfkarren/selene/releases/tag/0.4.0) - 2019-11-13
### Added
- A Roblox standard library can now be generated by simply having `std = "roblox"` in your configuration and running selene. If it does not exist, it will create one. This can also be initiated manually with `selene generate-roblox-std`.
- Added [`roblox_incorrect_color3_new_bounds`](https://kampfkarren.github.io/selene/lints/roblox_incorrect_color3_new_bounds.html).
- Added support for binary literals when using the `roblox` feature flag.

### Changed
- Changed incorrect_roact_usage to roblox_incorrect_roact_usage. [(#41)](https://github.com/Kampfkarren/selene/issues/41)
- Changed parsing errors to produce prettier results.

## [0.3.0](https://github.com/Kampfkarren/selene/releases/tag/0.3.0) - 2019-11-08
### Added
- Added `--color` option to specify whether colors could be used on the output.
- Added [`incorrect_roact_usage`](https://kampfkarren.github.io/selene/lints/incorrect_roact_usage.html) lint to verify correct usage of Roact.createElement.
- Added [`unscoped_variables`](https://kampfkarren.github.io/selene/lints/unscoped_variables.html) lint to disallow usage of unscoped (global) variables.

### Changed
- Colors will no longer be on by default when being piped. [(#32)](https://github.com/Kampfkarren/selene/issues/32)

### Fixed
- Fixed false positive with `unused_variable` linting function declarations as only mutations. [(#30)](https://github.com/Kampfkarren/selene/issues/30)
- Fixed terminal colors not resetting properly. [(#33)](https://github.com/Kampfkarren/selene/issues/33)

## [0.2.0](https://github.com/Kampfkarren/selene/releases/tag/0.2.0) - 2019-11-06
### Added
- Added standard library chaining. This means you can combine two standard libraries by setting `std` in selene.toml to `std1+std2`. You can chain as many as you want.

## [0.1.0](https://github.com/Kampfkarren/selene/releases/tag/0.1.0) - 2019-11-06
- Initial release
