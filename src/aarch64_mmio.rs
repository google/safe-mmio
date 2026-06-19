// Copyright 2025 The safe-mmio Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

use crate::backend::mmio_ops::MmioOps;
use crate::{SharedMmioPointer, UniqueMmioPointer};
use zerocopy::{FromBytes, Immutable, IntoBytes};

macro_rules! asm_mmio {
    ($t:ty, $read_name:ident, $read_assembly:literal, $write_name:ident, $write_assembly:literal) => {
        unsafe fn $read_name(ptr: *const $t) -> $t {
            let value;
            unsafe {
                core::arch::asm!(
                    $read_assembly,
                    value = out(reg) value,
                    ptr = in(reg) ptr,
                );
            }
            value
        }

        unsafe fn $write_name(ptr: *mut $t, value: $t) {
            unsafe {
                core::arch::asm!(
                    $write_assembly,
                    value = in(reg) value,
                    ptr = in(reg) ptr,
                );
            }
        }
    };
}

asm_mmio!(
    u8,
    read_u8,
    "ldrb {value:w}, [{ptr}]",
    write_u8,
    "strb {value:w}, [{ptr}]"
);
asm_mmio!(
    u16,
    read_u16,
    "ldrh {value:w}, [{ptr}]",
    write_u16,
    "strh {value:w}, [{ptr}]"
);
asm_mmio!(
    u32,
    read_u32,
    "ldr {value:w}, [{ptr}]",
    write_u32,
    "str {value:w}, [{ptr}]"
);
asm_mmio!(
    u64,
    read_u64,
    "ldr {value:x}, [{ptr}]",
    write_u64,
    "str {value:x}, [{ptr}]"
);

/// MmioOps backend using aarch64 inline assembly for MMIO access.
struct Ops;

// SAFETY: Each method delegates to the corresponding inline-assembly function of the matching width.
unsafe impl MmioOps for Ops {
    unsafe fn read_u8(src: *const u8) -> u8 {
        // SAFETY: Caller guarantees src is valid and aligned.
        unsafe { read_u8(src) }
    }

    unsafe fn read_u16(src: *const u16) -> u16 {
        // SAFETY: Caller guarantees src is valid and aligned.
        unsafe { read_u16(src) }
    }

    unsafe fn read_u32(src: *const u32) -> u32 {
        // SAFETY: Caller guarantees src is valid and aligned.
        unsafe { read_u32(src) }
    }

    unsafe fn read_u64(src: *const u64) -> u64 {
        // SAFETY: Caller guarantees src is valid and aligned.
        unsafe { read_u64(src) }
    }

    unsafe fn write_u8(dst: *mut u8, value: u8) {
        // SAFETY: Caller guarantees dst is valid and aligned.
        unsafe {
            write_u8(dst, value);
        }
    }

    unsafe fn write_u16(dst: *mut u16, value: u16) {
        // SAFETY: Caller guarantees dst is valid and aligned.
        unsafe {
            write_u16(dst, value);
        }
    }

    unsafe fn write_u32(dst: *mut u32, value: u32) {
        // SAFETY: Caller guarantees dst is valid and aligned.
        unsafe {
            write_u32(dst, value);
        }
    }

    unsafe fn write_u64(dst: *mut u64, value: u64) {
        // SAFETY: Caller guarantees dst is valid and aligned.
        unsafe {
            write_u64(dst, value);
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
