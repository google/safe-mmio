// Copyright 2025 The safe-mmio Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

use crate::backend::mmio_ops::MmioOps;

/// MmioOps backend using volatile read/write for MMIO access.
pub struct Ops;

// SAFETY: Each method performs a single volatile access of the indicated width.
unsafe impl MmioOps for Ops {
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
        unsafe {
            dst.write_volatile(value);
        }
    }

    unsafe fn write_u16(dst: *mut u16, value: u16) {
        // SAFETY: Caller guarantees dst is valid and aligned.
        unsafe {
            dst.write_volatile(value);
        }
    }

    unsafe fn write_u32(dst: *mut u32, value: u32) {
        // SAFETY: Caller guarantees dst is valid and aligned.
        unsafe {
            dst.write_volatile(value);
        }
    }

    unsafe fn write_u64(dst: *mut u64, value: u64) {
        // SAFETY: Caller guarantees dst is valid and aligned.
        unsafe {
            dst.write_volatile(value);
        }
    }
}
