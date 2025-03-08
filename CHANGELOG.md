# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.6.0] - 2025-03-08

### Added

- Tests for config deserialization
- Tests for checking and handling actions
- Support for unlimited threshold actions and notifications
- Implemented templating for notification texts
- Tests for notification templating

### Changed

- Improved config deserialization
- Refactored error handling to use `anyhow`
- Updated dependencies

### Removed

- Unused license from cargo-deny
- Support for explicit low/critical threshold notifications and commands

## [0.5.0] - 2025-02-18

### Added

- Project icon

### Changed

- Renamed project because of naming conflict on crates.io

## [0.4.0] - 2025-02-14

### Added

- Support for optional custom actions on low and critical thresholds

### Changed

- Update dependencies
- Update cargo deny config
- Update README
- Update example config
- Refactor config handling
- Make log messages more consistent

### Fixed

- Removed use of `unwrap`

## [0.3.4] - 2023-08-23

### Added

- README file

### Changed

- Update dependencies
- Update notification text

### Fixed

- Don't panic if notification fails

## [0.3.1] - 2023-02-25

### Added

- Logging

### Changed

- Truncate percentage in notification

## [0.2.0] - 2022-08-25

### Added

- Initial PoC

[Unreleased]: https://github.com/t4k1t/battered/compare/v0.6.0...HEAD
[0.6.0]: https://github.com/t4k1t/battered/compare/v0.6.0...v0.5.0
[0.5.0]: https://github.com/t4k1t/battered/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/t4k1t/battered/compare/v0.3.4...v0.4.0
[0.3.4]: https://github.com/t4k1t/battered/compare/v0.3.1...v0.3.4
[0.3.1]: https://github.com/t4k1t/battered/compare/v0.2.0...v0.3.1
[0.2.0]: https://github.com/t4k1t/battered/releases/tag/v0.2.0
