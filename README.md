# safe-mmio

[![crates.io page](https://img.shields.io/crates/v/safe-mmio.svg)](https://crates.io/crates/safe-mmio)
[![docs.rs page](https://docs.rs/safe-mmio/badge.svg)](https://docs.rs/safe-mmio)

This crate provides types for safe MMIO device access, especially in systems with an MMU.

This is not an officially supported Google product.

## Comparison with other MMIO crates

There are a number of things that distinguish this crate from other crates providing abstractions
for MMIO in Rust.

1. We avoid creating references to MMIO address space. The Rust compiler is free to dereference
   references whenever it likes, so constructing references to MMIO address space (even temporarily)
   can lead to undefined behaviour. See https://github.com/rust-embedded/volatile-register/issues/10
   for more background on this.
2. We distinguish between MMIO reads which have side-effects (e.g. clearing an interrupt status, or
   popping from a queue) and those which don't (e.g. just reading some status). A read which has
   side-effects should be treated like a write and only be allowed from a unique pointer (passed via
   &mut) whereas a read without side-effects can safely be done via a shared pointer (passed via
   '&'), e.g. simultaneously from multiple threads.
3. On most platforms MMIO reads and writes can be done via `read_volatile` and `write_volatile`, but
   on aarch64 this may generate instructions which can't be virtualised. This is arguably
   [a bug in rustc](https://github.com/rust-lang/rust/issues/131894), but in the meantime we work
   around this by using inline assembly to generate the correct instructions for MMIO reads and
   writes on aarch64.

| Crate name                                                      | Last release   | Version | Avoids references | Distinguishes reads with side-effects | Works around aarch64 volatile bug | Model                               | Field projection                     | Notes                                                                             |
| --------------------------------------------------------------- | -------------- | ------- | ----------------- | ------------------------------------- | --------------------------------- | ----------------------------------- | ------------------------------------ | --------------------------------------------------------------------------------- |
| safe-mmio                                                       | unreleased     | 0.2.0   | ✅                | ✅                                    | ✅                                | struct with field wrappers          | macro                                |
| [derive-mmio](https://crates.io/crates/derive-mmio)             | February 2025  | 0.3.0   | ✅                | ❌                                    | ❌                                | struct with derive macro            | only one level, through derive macro |
| [volatile](https://crates.io/crates/volatile)                   | June 2024      | 0.6.1   | ✅                | ❌                                    | ❌                                | struct with derive macro            | macro or generated methods           |
| [volatile-register](https://crates.io/crates/volatile-register) | October 2023   | 0.2.2   | ❌                | ❌                                    | ❌                                | struct with field wrappers          | manual (references)                  |
| [tock-registers](https://crates.io/crates/tock-registers)       | September 2023 | 0.9.0   | ❌                | ❌                                    | ❌                                | macros to define fields and structs | manual (references)                  | Also covers CPU registers, and bitfields                                          |
| [mmio](https://crates.io/crates/mmio)                           | May 2021       | 2.1.0   | ✅                | ❌                                    | ❌                                | only deals with individual fields   | ❌                                   |
| [rumio](https://crates.io/crates/rumio)                         | March 2021     | 0.2.0   | ✅                | ❌                                    | ❌                                | macros to define fields and structs | generated methods                    | Also covers CPU registers, and bitfields                                          |
| [vcell](https://crates.io/crates/vcell)                         | January 2021   | 0.1.3   | ❌                | ❌                                    | ❌                                | plain struct                        | manual (references)                  |
| [register](https://crates.io/crates/register)                   | January 2021   | 1.0.2   | ❌                | ❌                                    | ❌                                | macros to define fields and structs | manual (references)                  | Deprecated in favour of tock-registers. Also covers CPU registers, and bitfields. |

## License

Licensed under either of

- Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

If you want to contribute to the project, see details of
[how we accept contributions](CONTRIBUTING.md).
