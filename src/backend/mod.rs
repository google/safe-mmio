pub mod mmio_ops;

#[cfg(all(not(target_arch = "aarch64"), not(feature = "custom-mmio")))]
pub mod volatile_mmio;

#[cfg(all(target_arch = "aarch64", not(feature = "custom-mmio")))]
pub mod aarch64_mmio;

#[cfg(feature = "custom-mmio")]
pub mod custom_mmio;

#[cfg(all(target_arch = "aarch64", not(feature = "custom-mmio")))]
pub use aarch64_mmio::Ops;

#[cfg(feature = "custom-mmio")]
pub use custom_mmio::Ops;

#[cfg(all(not(target_arch = "aarch64"), not(feature = "custom-mmio")))]
pub use volatile_mmio::Ops;
