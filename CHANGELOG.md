# Changelog
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased
### Fixed
- Fixed display style option triggering `ArgumentConflict` when using quiet option. [(#288)](https://github.com/Kampfkarren/selene/issues/288)
- `bad_string_escape` now correctly handles escapes of the shape `\1a` (one or two numbers followed by a hex digit). (#292)[https://github.com/Kampfkarren/selene/issues/292]

## [0.14.0] - 2021-07-07
### Added
- Added `task` library to Roblox standard library.

### Changed
- `mismatched_arg_count` now tries to find the best overlap between multiple definitions, rather than ignoring them entirely. This means that if you have `f(a)` and `f(b, c)` defined, then calling `f(1, 2, 3)` will now lint instead of silently passing, since no definition provided meets it.
- `mismatched_arg_count` now shows all function definitions, rather than the local variable assignment. [(#259)](https://github.com/Kampfkarren/selene/issues/259)

### Fixed
- Updated internal parser, fixing some bugs with Roblox parsing. [See the changelog here](https://github.com/Kampfkarren/full-moon/blob/main/CHANGELOG.md#0131---2021-07-07).

## [0.13.0] - 2021-07-01
### Added
- Added `debug.info` to the Roblox standard library. [(#260)](https://github.com/Kampfkarren/selene/issues/260)
- Tokenization errors now form rich diagnostics.

### Changed
- Updated internal parser.
- Optimized linting process to run better with multiple threads.

### Fixed
- Fixed internal selene panics exiting with a zero code. Panics will now exit with status code 1, allowing it to be picked up by CI.
- Fixed variables named `self` not showing as unused if `allow_unused_self` was enabled. The implicit `self` variable being unused will still respect this configuration. [(#215)](https://github.com/Kampfkarren/selene/issues/215)

## [0.12.1] - 2021-05-26
### Fixed
- Fixed compile warning about unused variables.

## [0.12.0] - 2021-05-26
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

## [0.11.0] - 2021-01-04
### Added
- Added support for DateTime in the Roblox standard library.
- Added support for `table.clear` in the Roblox standard library.

## [0.10.1] - 2020-12-22
### Fixed
- Fixed regressions related to numeric for loops.

## [0.10.0] - 2020-12-21
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

## [0.9.2] - 2020-11-06
### Changed
- Updated full-moon, read [the full-moon changelog](https://github.com/Kampfkarren/full-moon/blob/master/CHANGELOG.md#070---2020-11-06) to learn more.

## [0.9.1] - 2020-11-04
### Fixed
- Fixed `--display-style=json` giving an output incompatible with previous tooling.

## [0.9.0] - 2020-11-04
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

## [0.8.0] - 2020-08-24
### Added
- Added support for `os.clock`.
- Added `RaycastParams.new`.
- Added support for `string.pack`, `string.packsize`, and `string.unpack` to the Roblox standard library.
- Added lint `compare_nan` to guard against comparing directly to nan (e.g. `x ~= 0/0`).
- Add lint `bad_string_escape` to guard invalid or malformed string escape sequences.

### Fixed
- Fixed `coroutine.yield` only accepting coroutines as a first argument.

## [0.7.0] - 2020-06-08
### Added
- Added support for `continue`, compound assignments (`+`), intersectional types, and numbers with underscores under the `roblox` feature flag.

### Fixed
- Fixed several parse errors with numbers.

### Changed
- Removed types from `debug.traceback` arguments in the Lua 5.1 standard library
- Made 4th argument to `CFrame.fromMatrix` optional (#113)
- Made standard library aware that functions and `...` can return multiple values

## [0.6.0] - 2020-04-21
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

## [0.5.3] - 2020-01-27
### Added
- Added tuple argument to `xpcall`
- Added `CFrame.fromEulerAnglesYXZ` to Roblox standard library
- Added `ColorSequenceKeypoint` to the Roblox standard library

### Fixed
- Fixed comments with tabs reporting as a parse error.

## [0.5.2] - 2020-01-19
### Fixed
- Fixed debug output for the standard library.

## [0.5.1] - 2020-01-19
### Added
- Added the `utf8` library to the Roblox standard library
- Added support for Typed Lua when using the Roblox feature flag.

### Changed
- Updated full-moon, which should result in faster parsing.

## [0.5.0] - 2019-12-20
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

## [0.4.3] - 2019-11-20
### Added
- Added `display-style` flag to use either rich (default), quiet (equivalent to -q), or JSON.

### Fixed
- Fixed a concatenated result always triggering an error when the standard library function expected a constant string (such as `collectgarbage` or `Instance.new`).
- Fixed parenthese conditions mixed with non-parenthese conditions (such as `(condition) and condition`) tripping the `parenthese_conditions` lint.

## [0.4.2] - 2019-11-13
### Fixed
- Fixed Roblox standard library not including structs, and thus failing when using `game`, `script`, etc.

## [0.4.1] - 2019-11-13
### Fixed
- Fixed Roblox standard library not including Lua 5.1 globals the first time you ran selene.

## [0.4.0] - 2019-11-13
### Added
- A Roblox standard library can now be generated by simply having `std = "roblox"` in your configuration and running selene. If it does not exist, it will create one. This can also be initiated manually with `selene generate-roblox-std`.
- Added [`roblox_incorrect_color3_new_bounds`](https://kampfkarren.github.io/selene/lints/roblox_incorrect_color3_new_bounds.html).
- Added support for binary literals when using the `roblox` feature flag.

### Changed
- Changed incorrect_roact_usage to roblox_incorrect_roact_usage. [(#41)](https://github.com/Kampfkarren/selene/issues/41)
- Changed parsing errors to produce prettier results.

## [0.3.0] - 2019-11-08
### Added
- Added `--color` option to specify whether colors could be used on the output.
- Added [`incorrect_roact_usage`](https://kampfkarren.github.io/selene/lints/incorrect_roact_usage.html) lint to verify correct usage of Roact.createElement.
- Added [`unscoped_variables`](https://kampfkarren.github.io/selene/lints/unscoped_variables.html) lint to disallow usage of unscoped (global) variables.

### Changed
- Colors will no longer be on by default when being piped. [(#32)](https://github.com/Kampfkarren/selene/issues/32)

### Fixed
- Fixed false positive with `unused_variable` linting function declarations as only mutations. [(#30)](https://github.com/Kampfkarren/selene/issues/30)
- Fixed terminal colors not resetting properly. [(#33)](https://github.com/Kampfkarren/selene/issues/33)

## [0.2.0] - 2019-11-06
### Added
- Added standard library chaining. This means you can combine two standard libraries by setting `std` in selene.toml to `std1+std2`. You can chain as many as you want.

## [0.1.0] - 2019-11-06
- Initial release
