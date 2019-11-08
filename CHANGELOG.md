# Changelog
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

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
