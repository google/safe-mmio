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

use zerocopy::{FromBytes, Immutable, IntoBytes};

use crate::{SharedMmioPointer, UniqueMmioPointer};
use core::mem::size_of;
use core::ptr::NonNull;

/// Trait for custom MMIO read/write implementations.
///
/// Per-size methods are used because MMIO access width matters at the hardware level (e.g. the
/// GHCB protocol needs to know the exact access size for VMGEXIT calls).
///
/// # Safety
///
/// Implementations must perform a single MMIO access of the indicated width at the given address.
/// The pointer is guaranteed to be properly aligned for its type and to point to valid MMIO address
/// space.
pub unsafe trait MmioOps {
    /// Perform an 8-bit MMIO read.
    ///
    /// # Safety
    ///
    /// `src` must be a valid, aligned pointer to MMIO address space.
    unsafe fn read_u8(src: *const u8) -> u8;

    /// Perform a 16-bit MMIO read.
    ///
    /// # Safety
    ///
    /// `src` must be a valid, aligned pointer to MMIO address space.
    unsafe fn read_u16(src: *const u16) -> u16;

    /// Perform a 32-bit MMIO read.
    ///
    /// # Safety
    ///
    /// `src` must be a valid, aligned pointer to MMIO address space.
    unsafe fn read_u32(src: *const u32) -> u32;

    /// Perform a 64-bit MMIO read.
    ///
    /// # Safety
    ///
    /// `src` must be a valid, aligned pointer to MMIO address space.
    unsafe fn read_u64(src: *const u64) -> u64;

    /// Perform an 8-bit MMIO write.
    ///
    /// # Safety
    ///
    /// `dst` must be a valid, aligned pointer to MMIO address space.
    unsafe fn write_u8(dst: *mut u8, value: u8);

    /// Perform a 16-bit MMIO write.
    ///
    /// # Safety
    ///
    /// `dst` must be a valid, aligned pointer to MMIO address space.
    unsafe fn write_u16(dst: *mut u16, value: u16);

    /// Perform a 32-bit MMIO write.
    ///
    /// # Safety
    ///
    /// `dst` must be a valid, aligned pointer to MMIO address space.
    unsafe fn write_u32(dst: *mut u32, value: u32);

    /// Perform a 64-bit MMIO write.
    ///
    /// # Safety
    ///
    /// `dst` must be a valid, aligned pointer to MMIO address space.
    unsafe fn write_u64(dst: *mut u64, value: u64);
}

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
        unsafe { mmio_read(self.regs) }
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
        // SAFETY: self.regs is always a valid and unique pointer to MMIO address space. The
        // extern functions are provided by the consumer via set_mmio_ops!().
        unsafe {
            match size_of::<T>() {
                1 => __safe_mmio_write_u8(self.regs.cast().as_ptr(), value.as_bytes()[0]),
                2 => __safe_mmio_write_u16(self.regs.cast().as_ptr(), convert(value)),
                4 => __safe_mmio_write_u32(self.regs.cast().as_ptr(), convert(value)),
                8 => __safe_mmio_write_u64(self.regs.cast().as_ptr(), convert(value)),
                _ => write_slice(self.regs.cast(), value.as_bytes()),
            }
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
        unsafe { mmio_read(self.regs) }
    }
}

/// Performs an MMIO read and returns the value.
///
/// # Safety
///
/// The pointer must be valid to perform an MMIO read from.
unsafe fn mmio_read<T: FromBytes + IntoBytes>(ptr: NonNull<T>) -> T {
    // SAFETY: ptr is a valid, aligned pointer to MMIO address space. The extern functions are
    // provided by the consumer via set_mmio_ops!(). For sizes 1/2/4/8 we perform a single
    // access; for larger sizes we split into chunks. The MaybeUninit is fully initialized before
    // calling assume_init().
    unsafe {
        match size_of::<T>() {
            1 => convert(__safe_mmio_read_u8(ptr.cast().as_ptr())),
            2 => convert(__safe_mmio_read_u16(ptr.cast().as_ptr())),
            4 => convert(__safe_mmio_read_u32(ptr.cast().as_ptr())),
            8 => convert(__safe_mmio_read_u64(ptr.cast().as_ptr())),
            _ => {
                let mut value = T::new_zeroed();
                read_slice(ptr.cast(), value.as_mut_bytes());
                value
            }
        }
    }
}

fn convert<T: Immutable + IntoBytes, U: FromBytes>(value: T) -> U {
    U::read_from_bytes(value.as_bytes()).unwrap()
}

/// # Safety
///
/// `ptr` must be valid for MMIO writes spanning `slice.len()` bytes.
unsafe fn write_slice(ptr: NonNull<u8>, slice: &[u8]) {
    if let Some((first, rest)) = slice.split_at_checked(8) {
        // SAFETY: Caller guarantees ptr is valid for the full slice length.
        unsafe {
            __safe_mmio_write_u64(ptr.cast().as_ptr(), u64::read_from_bytes(first).unwrap());
            write_slice(ptr.add(8), rest);
        }
    } else if let Some((first, rest)) = slice.split_at_checked(4) {
        // SAFETY: Caller guarantees ptr is valid for the full slice length.
        unsafe {
            __safe_mmio_write_u32(ptr.cast().as_ptr(), u32::read_from_bytes(first).unwrap());
            write_slice(ptr.add(4), rest);
        }
    } else if let Some((first, rest)) = slice.split_at_checked(2) {
        // SAFETY: Caller guarantees ptr is valid for the full slice length.
        unsafe {
            __safe_mmio_write_u16(ptr.cast().as_ptr(), u16::read_from_bytes(first).unwrap());
            write_slice(ptr.add(2), rest);
        }
    } else if let [first, rest @ ..] = slice {
        // SAFETY: Caller guarantees ptr is valid for the full slice length.
        unsafe {
            __safe_mmio_write_u8(ptr.as_ptr(), *first);
            write_slice(ptr.add(1), rest);
        }
    }
}

/// # Safety
///
/// `ptr` must be valid for MMIO reads spanning `slice.len()` bytes.
unsafe fn read_slice(ptr: NonNull<u8>, slice: &mut [u8]) {
    if let Some((first, rest)) = slice.split_at_mut_checked(8) {
        // SAFETY: Caller guarantees ptr is valid for the full slice length.
        unsafe {
            __safe_mmio_read_u64(ptr.cast().as_ptr())
                .write_to(first)
                .unwrap();
            read_slice(ptr.add(8), rest);
        }
    } else if let Some((first, rest)) = slice.split_at_mut_checked(4) {
        // SAFETY: Caller guarantees ptr is valid for the full slice length.
        unsafe {
            __safe_mmio_read_u32(ptr.cast().as_ptr())
                .write_to(first)
                .unwrap();
            read_slice(ptr.add(4), rest);
        }
    } else if let Some((first, rest)) = slice.split_at_mut_checked(2) {
        // SAFETY: Caller guarantees ptr is valid for the full slice length.
        unsafe {
            __safe_mmio_read_u16(ptr.cast().as_ptr())
                .write_to(first)
                .unwrap();
            read_slice(ptr.add(2), rest);
        }
    } else if let [first, rest @ ..] = slice {
        // SAFETY: Caller guarantees ptr is valid for the full slice length.
        unsafe {
            *first = __safe_mmio_read_u8(ptr.as_ptr());
            read_slice(ptr.add(1), rest);
        }
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
