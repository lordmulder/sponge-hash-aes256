# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Version 1.8.3

### Changed

- Updated `aes` dependency to version 0.9.0 rc-3 (2026-02-01).

## Version 1.8.2

### Added

- Added benchmarking utility to the command-line application. Run with `cargo bench --bench app_bench`.

### Changed

- Updated GitHub workflow (CI) to create builds with Rust version 1.93.0 (2026-01-22).
- Updated GitHub workflow (CI) to run the integration tests on the AARCH64 architecture too (Linux and Windows).

## Version 1.8.1

### Added

- Added option `--all` to process *all* files found in a directory (instead of "regular" files only).

### Changed

- The `--dirs` and `--recursive` options now skip non-regular files (such as FIFOs and sockets) by default.
- Various small improvements.

## Version 1.8.0

### Added

- Implemented multi-threaded processing of files in the command-line application &#x1F680;
- Added a “workspace” manifest (`Cargo.toml`) to the root of the repository to simplify the build process.
- Added support for [`MiMalloc`](https://crates.io/crates/mimalloc) allocator to the command-line application.
- Added code coverage analysis, via [Codecov.io](https://codecov.io/), and vastly improved the code coverage of our tests.

### Changed

- Minor performance improvements in the “core” library (yet again).
- Code clean-up all over the place.
- Updated GitHub workflow (CI) to create builds with Rust version 1.92.0 (2025-12-11).

## Version 1.7.0

### Added

- Added additional unit tests and added new benchmarks for the `SpongeHash256` struct.
- Added `SpongeHash256::update_range()` function to process a range specified by two “raw” pointers.

### Changed

- Some additional performance improvements in the “core” library (again).

## Version 1.6.1

### Changed

- Some additional performance improvements in the “core” library.

## Version 1.6.0

### Added

- Added dedicated i686 and x86-64 builds with AES-NI enabled to the GitHub CI workflow.

### Changed

- Various significant performance improvements in the "core" library.
- Updated `aes` from version 0.8.4 to version 0.9.0 (rc-2) and dropped `generic_array` dependency.
- Updated GitHub workflow (CI) to create builds with Rust version 1.91.1 (2025-11-10).

### Removed

- Removed `wide` feature, as the `wide` dependency is now always enabled.

## Version 1.5.1

### Added

- Added Windows 7–compatible Windows builds to the GitHub CI workflow.

### Changed

- Updated GitHub workflow (CI) to create builds with Rust version 1.91.0 (2025-10-30).

### Fixed

- Fixed a performance regression in version 1.5.0.

## Version 1.5.0

### Added

- Added an environment variable to switch between `BFS` and `DFS` search strategies.
- Updated GitHub workflow (CI) to produce automated builds for the DragonFly BSD platform.

### Changed

- Implemented an improved memory handling for the 128-bit AES blocks and the 256-bit key data.
- Implemented faster methods for XOR'ing data blocks and concatenating key data.
- Lowered the required Rust edition to 2021.
- Implemented workarounds to allow building with older Rust versions (1.78.0 or newer).
- Implemented an improved method to shut down the application when receiving a `SIGINT` signal.
- Various small improvements.

## Version 1.4.2

### Added

- Implemented breadth-first search (BFS) for recursively iterating a directory tree.
- Implemented detection of file system loops (Unix only).

### Changed

- Various small improvements.

## Version 1.4.1

### Fixed

- Fixed GitHub workflow (CI) for Linux to actually apply the `Ctarget-cpu` option as intended.

## Version 1.4.0

### Added

- Implemented support for option `--check` to verify files from an existing checksum file.
- Updated GitHub workflow (CI) to produce a `.deb` installation package for Linux.

### Changed

- New workaround for deprecation of `generic_array` version 0.14.x, now using version 1.3.x.
- Various improvements to the self-test code.
- Various improvements to the Windows (Nullsoft) installer.

## Version 1.3.5

### Added

- Implemented workaround for the deprecation of `generic_array` in version 0.14.8+.

### Removed

- Feature `aligned` has been removed. Now enabled implicitly when building in "release" mode.

### Changed

- Updated GitHub workflow (CI) to run the tests on Windows and macOS, in addition to Linux.
- Various improvements to the Windows (Nullsoft) installer.

## Version 1.3.4

### Added

- Updated GitHub workflow (CI) to produce automated builds for the OpenBSD, Solaris and Illumos platforms.

### Changed

- Switched to [VM Actions](https://github.com/vmactions) for creating the automated builds for the Unix/BSD platforms.
- Reduced the size of the documentation files that are included in the generated release bundles.
- Various small improvements.

## Version 1.3.3

### Added

- Added new options `--dirs` and `--recursive` to the command-line application.
- Added new option `--null` to the command-line application.

### Changed

- The build scripts (Makefiles) for various Unix/BSD platforms have been unified.
- Various small improvements.

## Version 1.3.2

### Changed

- Implemented some workarounds to make the code compile with older `rustc` versions.
- Various small improvements.

### Fixed

- Re-enabled `#![no_std]` flag for the library crate (regression in 1.3.1).
- Fixed a possible alignment issues in `xor_arrays()`.

## Version 1.3.1

### Added

- Added new option `--self-test` to the command-line application.
- Updated GitHub workflow (CI) to produce automated builds for NetBSD too.

### Changed

- Various small improvements.

## Version 1.3.0

### Added

- Added a fully-featured command-line application (`sponge256sum`).
- Updated GitHub workflow (CI) to produce automated builds for Linux, macOS, FreeBSD and Windows.

### Changed

- Some tweaks to the hash algorithm.
