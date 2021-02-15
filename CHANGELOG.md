# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- Added a `free` method.
- Expanded all examples.

### Changed
- Moved instruction constants to an `instruction` module.
- Changed `MAX_ADDR` from a `usize` to a `u8`.
- Changed `PAGE_SIZE` from a `usize` to a `u8`.
- Renamed `read_data` to `read`.
- `read_eui48` now returns `[u8; 6]` instead of accepting it as a mutable buffer.

### Fixed
- Fixed writes not working due to the write latch being disabled.

### Removed
- Removed the `write_byte` method.
- Removed the `MAX_ADDR` constant.

## [0.1.0] - 2020-09-12
- Initial release

[Unreleased]: https://github.com/newAM/eeprom25aa02e48-rs/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/newAM/eeprom25aa02e48-rs/releases/tag/v0.1.0
