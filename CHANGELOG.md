# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
