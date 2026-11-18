# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Add `container_as_blob` ([#63](https://github.com/monero-rs/monero-epee-bin-serde/pull/63))

### Changed

- Allow optional values ([#59](https://github.com/monero-rs/monero-epee-bin-serde/pull/59))
- Updated Rust version to 2021 and bumped MSRV to 1.63 ([#52](https://github.com/monero-rs/monero-epee-bin-serde/pull/52)).
- `from_bytes` rejects input with bytes remaining after the root object.

### Fixed

- Fixed serialization of empty sequences ([#58](https://github.com/monero-rs/monero-epee-bin-serde/pull/58))
- No longer panic on unknown fields ([#46](https://github.com/monero-rs/monero-epee-bin-serde/pull/46))
- Deserialization of nested structs ([#37](https://github.com/monero-rs/monero-epee-bin-serde/pull/37)).
- No longer panic when the input ends where a length is expected, and no longer truncate lengths which do not fit into `usize` on 32-bit targets.
- No longer allocate the declared length of a string or byte array before its bytes have been read.
- No longer overflow the stack on deeply nested input. Objects and sequences nested more than 64 levels deep are rejected.

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
