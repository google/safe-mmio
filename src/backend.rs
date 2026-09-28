// Copyright 2026 The safe-mmio Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

#[cfg(all(target_arch = "aarch64", not(feature = "custom-mmio"), not(kani)))]
pub mod aarch64;
#[cfg(feature = "custom-mmio")]
pub mod custom;
pub mod mmio_ops;
#[cfg(all(not(feature = "custom-mmio"), any(not(target_arch = "aarch64"), kani)))]
pub mod volatile;

#[cfg(all(target_arch = "aarch64", not(feature = "custom-mmio"), not(kani)))]
pub use aarch64::Ops;
#[cfg(feature = "custom-mmio")]
pub use custom::Ops;
#[cfg(all(not(feature = "custom-mmio"), any(not(target_arch = "aarch64"), kani)))]
pub use volatile::Ops;
