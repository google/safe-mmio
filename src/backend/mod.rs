#[cfg(any(target_arch = "aarch64", feature = "custom-mmio"))]
pub mod mmio_ops;

#[cfg(all(not(target_arch = "aarch64"), not(feature = "custom-mmio")))]
pub mod volatile_mmio;

#[cfg(all(target_arch = "aarch64", not(feature = "custom-mmio")))]
pub mod aarch64_mmio;

#[cfg(feature = "custom-mmio")]
pub mod custom_mmio;
