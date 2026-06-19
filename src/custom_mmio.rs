// Copyright 2026 The safe-mmio Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

//! Custom MMIO backend using consumer-provided read/write implementations.
//!
//! When the `custom-mmio` feature is enabled, the consumer must provide an implementation of the
//! [`MmioOps`] trait and register it using the [`set_mmio_ops!`](crate::set_mmio_ops) macro.
//! Linking will fail if no implementation is registered.
//!
//! # Example
//!
//! ```
//! use safe_mmio::{custom_mmio::MmioOps, set_mmio_ops};
//!
//! struct MyMmioBackend;
//!
//! unsafe impl MmioOps for MyMmioBackend {
//!     unsafe fn read_u8(src: *const u8) -> u8 {
//!         src.read_volatile()
//!     }
//!     unsafe fn read_u16(src: *const u16) -> u16 {
//!         src.read_volatile()
//!     }
//!     unsafe fn read_u32(src: *const u32) -> u32 {
//!         src.read_volatile()
//!     }
//!     unsafe fn read_u64(src: *const u64) -> u64 {
//!         src.read_volatile()
//!     }
//!     unsafe fn write_u8(dst: *mut u8, value: u8) {
//!         dst.write_volatile(value);
//!     }
//!     unsafe fn write_u16(dst: *mut u16, value: u16) {
//!         dst.write_volatile(value);
//!     }
//!     unsafe fn write_u32(dst: *mut u32, value: u32) {
//!         dst.write_volatile(value);
//!     }
//!     unsafe fn write_u64(dst: *mut u64, value: u64) {
//!         dst.write_volatile(value);
//!     }
//! }
//!
//! set_mmio_ops!(MyMmioBackend);
//! ```
pub use crate::backend::mmio_ops::MmioOps;
use crate::{SharedMmioPointer, UniqueMmioPointer};
use zerocopy::{FromBytes, Immutable, IntoBytes};

unsafe extern "Rust" {
    fn __safe_mmio_read_u8(src: *const u8) -> u8;
    fn __safe_mmio_read_u16(src: *const u16) -> u16;
    fn __safe_mmio_read_u32(src: *const u32) -> u32;
    fn __safe_mmio_read_u64(src: *const u64) -> u64;
    fn __safe_mmio_write_u8(dst: *mut u8, value: u8);
    fn __safe_mmio_write_u16(dst: *mut u16, value: u16);
    fn __safe_mmio_write_u32(dst: *mut u32, value: u32);
    fn __safe_mmio_write_u64(dst: *mut u64, value: u64);
}

/// Register a [`MmioOps`] implementation as the MMIO backend.
///
/// This macro must be called exactly once in the final binary when the `custom-mmio` feature is
/// enabled. It generates the linker symbols that bridge the [`MmioOps`] trait to the internal
/// extern function declarations.
///
/// # Example
///
/// ```ignore
/// use safe_mmio::set_mmio_ops;
///
/// struct MyMmioBackend;
///
/// set_mmio_ops!(MyMmioBackend);
/// ```
#[macro_export]
macro_rules! set_mmio_ops {
    ($t:ty) => {
        #[unsafe(no_mangle)]
        unsafe fn __safe_mmio_read_u8(src: *const u8) -> u8 {
            // SAFETY: Caller guarantees src is valid and aligned for MMIO.
            unsafe { <$t as $crate::custom_mmio::MmioOps>::read_u8(src) }
        }

        #[unsafe(no_mangle)]
        unsafe fn __safe_mmio_read_u16(src: *const u16) -> u16 {
            // SAFETY: Caller guarantees src is valid and aligned for MMIO.
            unsafe { <$t as $crate::custom_mmio::MmioOps>::read_u16(src) }
        }

        #[unsafe(no_mangle)]
        unsafe fn __safe_mmio_read_u32(src: *const u32) -> u32 {
            // SAFETY: Caller guarantees src is valid and aligned for MMIO.
            unsafe { <$t as $crate::custom_mmio::MmioOps>::read_u32(src) }
        }

        #[unsafe(no_mangle)]
        unsafe fn __safe_mmio_read_u64(src: *const u64) -> u64 {
            // SAFETY: Caller guarantees src is valid and aligned for MMIO.
            unsafe { <$t as $crate::custom_mmio::MmioOps>::read_u64(src) }
        }

        #[unsafe(no_mangle)]
        unsafe fn __safe_mmio_write_u8(dst: *mut u8, value: u8) {
            // SAFETY: Caller guarantees dst is valid and aligned for MMIO.
            unsafe { <$t as $crate::custom_mmio::MmioOps>::write_u8(dst, value) }
        }

        #[unsafe(no_mangle)]
        unsafe fn __safe_mmio_write_u16(dst: *mut u16, value: u16) {
            // SAFETY: Caller guarantees dst is valid and aligned for MMIO.
            unsafe { <$t as $crate::custom_mmio::MmioOps>::write_u16(dst, value) }
        }

        #[unsafe(no_mangle)]
        unsafe fn __safe_mmio_write_u32(dst: *mut u32, value: u32) {
            // SAFETY: Caller guarantees dst is valid and aligned for MMIO.
            unsafe { <$t as $crate::custom_mmio::MmioOps>::write_u32(dst, value) }
        }

        #[unsafe(no_mangle)]
        unsafe fn __safe_mmio_write_u64(dst: *mut u64, value: u64) {
            // SAFETY: Caller guarantees dst is valid and aligned for MMIO.
            unsafe { <$t as $crate::custom_mmio::MmioOps>::write_u64(dst, value) }
        }
    };
}

/// MmioOps backend delegating to consumer-provided extern functions.
struct Ops;

// SAFETY: Each method delegates to the consumer-provided extern function of the matching width.
unsafe impl MmioOps for Ops {
    unsafe fn read_u8(src: *const u8) -> u8 {
        // SAFETY: Caller guarantees src is valid and aligned.
        unsafe { __safe_mmio_read_u8(src) }
    }

    unsafe fn read_u16(src: *const u16) -> u16 {
        // SAFETY: Caller guarantees src is valid and aligned.
        unsafe { __safe_mmio_read_u16(src) }
    }

    unsafe fn read_u32(src: *const u32) -> u32 {
        // SAFETY: Caller guarantees src is valid and aligned.
        unsafe { __safe_mmio_read_u32(src) }
    }

    unsafe fn read_u64(src: *const u64) -> u64 {
        // SAFETY: Caller guarantees src is valid and aligned.
        unsafe { __safe_mmio_read_u64(src) }
    }

    unsafe fn write_u8(dst: *mut u8, value: u8) {
        // SAFETY: Caller guarantees dst is valid and aligned.
        unsafe {
            __safe_mmio_write_u8(dst, value);
        }
    }

    unsafe fn write_u16(dst: *mut u16, value: u16) {
        // SAFETY: Caller guarantees dst is valid and aligned.
        unsafe {
            __safe_mmio_write_u16(dst, value);
        }
    }

    unsafe fn write_u32(dst: *mut u32, value: u32) {
        // SAFETY: Caller guarantees dst is valid and aligned.
        unsafe {
            __safe_mmio_write_u32(dst, value);
        }
    }

    unsafe fn write_u64(dst: *mut u64, value: u64) {
        // SAFETY: Caller guarantees dst is valid and aligned.
        unsafe {
            __safe_mmio_write_u64(dst, value);
        }
    }
}

impl<T: FromBytes + IntoBytes> UniqueMmioPointer<'_, T> {
    /// Performs an MMIO read and returns the value.
    ///
    /// If `T` is exactly 1, 2, 4 or 8 bytes long then this will be a single operation. Otherwise
    /// it will be split into several, reading chunks as large as possible.
    ///
    /// Note that this takes `&mut self` rather than `&self` because an MMIO read may cause
    /// side-effects that change the state of the device.
    ///
    /// # Safety
    ///
    /// This field must be safe to perform an MMIO read from.
    pub unsafe fn read_unsafe(&mut self) -> T {
        // SAFETY: self.regs is always a valid and unique pointer to MMIO address space.
        unsafe { Ops::mmio_read(self.regs) }
    }
}

impl<T: Immutable + IntoBytes> UniqueMmioPointer<'_, T> {
    /// Performs an MMIO write of the given value.
    ///
    /// If `T` is exactly 1, 2, 4 or 8 bytes long then this will be a single operation. Otherwise
    /// it will be split into several, writing chunks as large as possible.
    ///
    /// # Safety
    ///
    /// This field must be safe to perform an MMIO write to.
    pub unsafe fn write_unsafe(&mut self, value: T) {
        // SAFETY: self.regs is always a valid and unique pointer to MMIO address space.
        unsafe {
            Ops::mmio_write(self.regs, value);
        }
    }
}

impl<T: FromBytes + IntoBytes> SharedMmioPointer<'_, T> {
    /// Performs an MMIO read and returns the value.
    ///
    /// If `T` is exactly 1, 2, 4 or 8 bytes long then this will be a single operation. Otherwise
    /// it will be split into several, reading chunks as large as possible.
    ///
    /// # Safety
    ///
    /// This field must be safe to perform an MMIO read from, and doing so must not cause any
    /// side-effects.
    pub unsafe fn read_unsafe(&self) -> T {
        // SAFETY: self.regs is always a valid pointer to MMIO address space.
        unsafe { Ops::mmio_read(self.regs) }
    }
}

#[cfg(test)]
mod testing {
    /// Default MMIO implementation using volatile access for unit tests.
    struct VolatileOps;

    // SAFETY: Each method performs a single volatile access of the indicated width.
    unsafe impl super::MmioOps for VolatileOps {
        unsafe fn read_u8(src: *const u8) -> u8 {
            // SAFETY: Caller guarantees src is valid and aligned.
            unsafe { src.read_volatile() }
        }

        unsafe fn read_u16(src: *const u16) -> u16 {
            // SAFETY: Caller guarantees src is valid and aligned.
            unsafe { src.read_volatile() }
        }

        unsafe fn read_u32(src: *const u32) -> u32 {
            // SAFETY: Caller guarantees src is valid and aligned.
            unsafe { src.read_volatile() }
        }

        unsafe fn read_u64(src: *const u64) -> u64 {
            // SAFETY: Caller guarantees src is valid and aligned.
            unsafe { src.read_volatile() }
        }

        unsafe fn write_u8(dst: *mut u8, value: u8) {
            // SAFETY: Caller guarantees dst is valid and aligned.
            unsafe { dst.write_volatile(value) }
        }

        unsafe fn write_u16(dst: *mut u16, value: u16) {
            // SAFETY: Caller guarantees dst is valid and aligned.
            unsafe { dst.write_volatile(value) }
        }

        unsafe fn write_u32(dst: *mut u32, value: u32) {
            // SAFETY: Caller guarantees dst is valid and aligned.
            unsafe { dst.write_volatile(value) }
        }

        unsafe fn write_u64(dst: *mut u64, value: u64) {
            // SAFETY: Caller guarantees dst is valid and aligned.
            unsafe { dst.write_volatile(value) }
        }
    }

    set_mmio_ops!(VolatileOps);
}
