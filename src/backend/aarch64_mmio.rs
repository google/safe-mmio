// Copyright 2025 The safe-mmio Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

use crate::backend::mmio_ops::MmioOps;

macro_rules! asm_mmio {
    ($t:ty, $read_name:ident, $read_assembly:literal, $write_name:ident, $write_assembly:literal) => {
        unsafe fn $read_name(ptr: *const $t) -> $t {
            let value;
            // SAFETY: Caller guarantees ptr is valid and aligned for the access width.
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
            // SAFETY: Caller guarantees ptr is valid and aligned for the access width.
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
pub struct Ops;

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
