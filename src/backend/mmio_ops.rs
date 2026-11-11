// Copyright 2026 The safe-mmio Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

use core::ptr::NonNull;
use zerocopy::{FromBytes, Immutable, IntoBytes};

fn convert<T: Immutable + IntoBytes, U: FromBytes>(value: T) -> U {
    U::read_from_bytes(value.as_bytes()).unwrap()
}

/// Trait for custom MMIO read/write implementations.
///
/// Per-size methods are used because MMIO access width matters at the hardware level (e.g. the
/// GHCB protocol needs to know the exact access size for VMGEXIT calls).
///
/// The pointer is guaranteed to be properly aligned for its type and to point to valid MMIO address
/// space.
pub trait MmioOps {
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

    /// Performs an MMIO read and returns the value.
    ///
    /// # Safety
    ///
    /// The pointer must be valid to perform an MMIO read from.
    unsafe fn read<T: FromBytes + IntoBytes>(ptr: NonNull<T>) -> T {
        // SAFETY: Caller guarantees `ptr` is valid for MMIO reads of `size_of::<T>()` bytes, and
        // the implementor provides correctly-functioning primitive MMIO operations. For naturally
        // aligned sizes 1/2/4/8 we perform a single access; otherwise we split into aligned chunks.
        unsafe {
            match size_of::<T>() {
                1 => convert(Self::read_u8(ptr.cast().as_ptr())),
                2 if ptr.cast::<u16>().is_aligned() => convert(Self::read_u16(ptr.cast().as_ptr())),
                4 if ptr.cast::<u32>().is_aligned() => convert(Self::read_u32(ptr.cast().as_ptr())),
                8 if ptr.cast::<u64>().is_aligned() => convert(Self::read_u64(ptr.cast().as_ptr())),
                _ => {
                    let mut value = T::new_zeroed();
                    Self::read_slice(ptr.cast(), value.as_mut_bytes());
                    value
                }
            }
        }
    }

    /// Reads from MMIO by splitting into naturally-sized chunks.
    ///
    /// # Safety
    ///
    /// `ptr` must be valid for MMIO reads spanning `slice.len()` bytes.
    unsafe fn read_slice(ptr: NonNull<u8>, slice: &mut [u8]) {
        if let Some((first, rest)) = slice.split_at_mut_checked(8)
            && ptr.cast::<u64>().is_aligned()
        {
            // SAFETY: Caller guarantees ptr is valid for the full slice length and we just checked
            // that it is properly aligned for u64.
            unsafe {
                Self::read_u64(ptr.cast().as_ptr()).write_to(first).unwrap();
                Self::read_slice(ptr.add(8), rest);
            }
        } else if let Some((first, rest)) = slice.split_at_mut_checked(4)
            && ptr.cast::<u32>().is_aligned()
        {
            // SAFETY: Caller guarantees ptr is valid for the full slice length and we just checked
            // that it is properly aligned for u32.
            unsafe {
                Self::read_u32(ptr.cast().as_ptr()).write_to(first).unwrap();
                Self::read_slice(ptr.add(4), rest);
            }
        } else if let Some((first, rest)) = slice.split_at_mut_checked(2)
            && ptr.cast::<u16>().is_aligned()
        {
            // SAFETY: Caller guarantees ptr is valid for the full slice length and we just checked
            // that it is properly aligned for u16.
            unsafe {
                Self::read_u16(ptr.cast().as_ptr()).write_to(first).unwrap();
                Self::read_slice(ptr.add(2), rest);
            }
        } else if let [first, rest @ ..] = slice {
            // SAFETY: Caller guarantees ptr is valid for the full slice length.
            unsafe {
                *first = Self::read_u8(ptr.as_ptr());
                Self::read_slice(ptr.add(1), rest);
            }
        }
    }

    /// Writes to MMIO by splitting into naturally-sized chunks.
    ///
    /// # Safety
    ///
    /// `ptr` must be valid for MMIO writes spanning `slice.len()` bytes.
    unsafe fn write_slice(ptr: NonNull<u8>, slice: &[u8]) {
        if let Some((first, rest)) = slice.split_at_checked(8)
            && ptr.cast::<u64>().is_aligned()
        {
            // SAFETY: Caller guarantees ptr is valid for the full slice length and we just checked
            // that it is properly aligned for u64.
            unsafe {
                Self::write_u64(ptr.cast().as_ptr(), u64::read_from_bytes(first).unwrap());
                Self::write_slice(ptr.add(8), rest);
            }
        } else if let Some((first, rest)) = slice.split_at_checked(4)
            && ptr.cast::<u32>().is_aligned()
        {
            // SAFETY: Caller guarantees ptr is valid for the full slice length and we just checked
            // that it is properly aligned for u32.
            unsafe {
                Self::write_u32(ptr.cast().as_ptr(), u32::read_from_bytes(first).unwrap());
                Self::write_slice(ptr.add(4), rest);
            }
        } else if let Some((first, rest)) = slice.split_at_checked(2)
            && ptr.cast::<u16>().is_aligned()
        {
            // SAFETY: Caller guarantees ptr is valid for the full slice length and we just checked
            // that it is properly aligned for u16.
            unsafe {
                Self::write_u16(ptr.cast().as_ptr(), u16::read_from_bytes(first).unwrap());
                Self::write_slice(ptr.add(2), rest);
            }
        } else if let [first, rest @ ..] = slice {
            // SAFETY: Caller guarantees ptr is valid for the full slice length.
            unsafe {
                Self::write_u8(ptr.as_ptr(), *first);
                Self::write_slice(ptr.add(1), rest);
            }
        }
    }

    /// Performs an MMIO write of the given value.
    ///
    /// # Safety
    ///
    /// `ptr` must be valid to perform an MMIO write to.
    unsafe fn write<T: Immutable + IntoBytes>(ptr: NonNull<T>, value: T) {
        // SAFETY: Caller guarantees `ptr` is valid for MMIO writes of `size_of::<T>()` bytes. For
        // naturally aligned sizes 1/2/4/8 we perform a single access; otherwise we split into
        // aligned chunks.
        unsafe {
            match size_of::<T>() {
                1 => Self::write_u8(ptr.cast().as_ptr(), value.as_bytes()[0]),
                2 if ptr.cast::<u16>().is_aligned() => {
                    Self::write_u16(ptr.cast().as_ptr(), convert(value));
                }
                4 if ptr.cast::<u32>().is_aligned() => {
                    Self::write_u32(ptr.cast().as_ptr(), convert(value));
                }
                8 if ptr.cast::<u64>().is_aligned() => {
                    Self::write_u64(ptr.cast().as_ptr(), convert(value));
                }
                _ => Self::write_slice(ptr.cast(), value.as_bytes()),
            }
        }
    }
}
