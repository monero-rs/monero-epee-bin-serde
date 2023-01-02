[![Build Status](https://img.shields.io/github/actions/workflow/status/comit-network/monero-epee-bin-serde/ci.yml?branch=main)](https://github.com/comit-network/monero-epee-bin-serde/actions/workflows/ci.yml)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)
[![Crates.io](https://img.shields.io/crates/v/monero-epee-bin-serde.svg)](https://crates.io/crates/monero-epee-bin-serde)
[![Documentation](https://docs.rs/monero-epee-bin-serde/badge.svg)](https://docs.rs/monero-epee-bin-serde)
[![License: MIT or Apache](https://img.shields.io/badge/License-MIT%20or%20Apache%202.0-yellow.svg)](./COPYRIGHT)

# `monero-epee-bin-serde`

This crate implements the binary encoding defined in the `epee` helper library of Monero [[0], [1]] as a [`serde`](https://docs.rs/serde) data format.

[0]: https://github.com/monero-project/monero/blob/0a1ddc2eff854f3e932203a95b65a9f1efd60eef/contrib/epee/include/storages/portable_storage_from_bin.h
[1]: https://github.com/monero-project/monero/blob/0a1ddc2eff854f3e932203a95b65a9f1efd60eef/contrib/epee/include/storages/portable_storage_to_bin.h

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
