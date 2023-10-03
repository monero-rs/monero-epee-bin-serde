# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- Updated Rust version to 2021 and bumped MSRV to 1.63 ([#52](https://github.com/monero-rs/monero-epee-bin-serde/pull/52)).

### Fixed

- No longer panic on unknown fields ([#46](https://github.com/monero-rs/monero-epee-bin-serde/pull/46))
- Deserialization of nested structs ([#37](https://github.com/monero-rs/monero-epee-bin-serde/pull/37)).

## [1.0.1] - 2021-07-09

### Fixed

- An issue where byte sequences were serialized as sequences instead of the expected byte string.

### Added

- Missing documentation on public items.
- Support for deserializing fixed-length byte arrays.

## [1.0.0] - 2021-04-30

Initial release.

[Unreleased]: https://github.com/monero-rs/monero-epee-bin-serde/compare/1.0.1...HEAD
[1.0.1]: https://github.com/monero-rs/monero-epee-bin-serde/compare/v1.0.0...1.0.1
[1.0.0]: https://github.com/comit-network/monero-epee-bin-serde/compare/f29ab8bbd9a7221fe921dc253ee9bf4f94e95f92...v1.0.0
