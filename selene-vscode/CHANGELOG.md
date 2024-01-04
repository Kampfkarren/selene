# Change Log

All notable changes to the "selene-vscode" extension will be documented in this file.

If you want to stay up to date with selene itself, you can find the changelog in [selene's CHANGELOG.md](https://github.com/Kampfkarren/selene/blob/master/CHANGELOG.md).

## Unreleased

## [1.5.1]
- selene now works with the Luau language ID.

## [1.5.0]
- Added `update-roblox-std` and `generate-roblox-std` subcommands to vscode command palette
- Added error reporting for configuration files, both for selene.toml and for YML standard libraries.

## [1.2.0]
- Temporary paths, such as Git diff previews, will now use your current workspace folder for configuration.
- Fixed a bug where the extension would become unusable if it couldn't connect to the internet.
- If a Roblox codebase is detected without a standard library, you will be prompted to set it up. You can disable this in the settings if you work in non-Roblox codebases that trip them up.

## [1.1.1]
- Fixed a bug that made new installations impossible.

## [1.1.0]
- Added `onSave`, `onType`, `onNewLint`, and `onIdle` configurations to choose when selene lints.
- Updated the internal `fs` code to use VSCode FileSystem rather than Node.js fs methods.
- Fixed some information (mainly parse error information) missing from diagnostics which were available in the selene CLI.

## [1.0.3]
- Fixed incorrect diagnostics positions when using Unicode characters

## [1.0.2]
- Fixed a bug where diagnostics would not be removed when a file was deleted.

## [1.0.1]
- Fixed a bug where diagnostics would stay left over.

## [1.0.0]
- Initial release