# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Version 1.3.2

### Changed

- Implemented some workarounds to make the code compile with older `rustc` versions.
- Various small improvements.

### Fixed

- Re-enabled `#![no_std]` flag for the library crate (regression in 1.3.1).
- Fixed a possible alignment issues in `xor_arrays()`.

## Version 1.3.1

### Added

- Added new option --self-test to the command-line application.
- Updated GitHub workflow (CI) to produce automated builds for NetBSD too.

### Changed

- Various small improvements.

## Version 1.3.0

### Added

- Added a fully-featured command-line application (`sponge256sum`).
- Updated GitHub workflow (CI) to produce automated builds for Linux, macOS, FreeBSD and Windows.

### Changed

- Some tweaks to the hash algorithm.
