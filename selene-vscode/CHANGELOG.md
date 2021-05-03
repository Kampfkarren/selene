# Change Log

All notable changes to the "selene-vscode" extension will be documented in this file.

If you want to stay up to date with selene itself, you can find the changelog in [selene's CHANGELOG.md](https://github.com/Kampfkarren/selene/blob/master/CHANGELOG.md).

## [Unreleased]
-   Add `onSave` and `onType` configuration to choose when selene lints.
-   Updated the internal `fs` code to use VSCode FileSystem rather than Node.js fs methods.
-   Fixed some information (mainly parse error information) missing from diagnostics which were available in the selene CLI.

## [1.0.3]
-   Fixed incorrect diagnostics positions when using Unicode characters

## [1.0.2]
-   Fixed a bug where diagnostics would not be removed when a file was deleted.

## [1.0.1]
-   Fixed a bug where diagnostics would stay left over.

## [1.0.0]
-   Initial release
