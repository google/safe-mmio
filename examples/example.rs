// Copyright 2026 The safe-mmio Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

//! Example demonstrating the use of the safe_mmio crate by accessing
//! a mock device (=regular RAM).
//!
//! This uses the default backend for the current architecture.
//!
//! Run with:
//!
//! ```sh
//! cargo run --example example
//! ```
//!
//! To enable the custom-mmio backend, run:
//! ```sh
//! cargo run --example example --features custom-mmio
//! ```
//! An [`MmioOps`] implementation is provided here which simply uses volatile
//! reads/writes.

use safe_mmio::fields::{ReadPure, ReadWrite};
use safe_mmio::{UniqueMmioPointer, field};

#[cfg(feature = "custom-mmio")]
mod custom_ops {
    struct VolatileOps;

    // SAFETY: Each method performs a single volatile access of the indicated width.
    unsafe impl safe_mmio::custom_mmio::MmioOps for VolatileOps {
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

    safe_mmio::set_mmio_ops!(VolatileOps);
}

#[repr(C)]
struct DeviceRegs {
    status: ReadPure<u32>,
    control: ReadWrite<u32>,
    data: ReadWrite<u64>,
}

fn main() {
    let mut regs = DeviceRegs {
        status: ReadPure(0x0000_00FF),
        control: ReadWrite(0),
        data: ReadWrite(0),
    };

    let mut ptr = UniqueMmioPointer::from(&mut regs);

    // Read a pure (side-effect-free) field via shared access.
    let status = field!(ptr, status).read();
    println!("status:  {status:#010x}");
    assert_eq!(status, 0xFF);

    // Write and read back a read-write field.
    field!(ptr, control).write(0xDEAD_BEEF);
    let control = field!(ptr, control).read();
    println!("control: {control:#010x}");
    assert_eq!(control, 0xDEAD_BEEF);

    // 64-bit read-write field.
    field!(ptr, data).write(0x0123_4567_89AB_CDEF);
    let data = field!(ptr, data).read();
    println!("data:    {data:#018x}");
    assert_eq!(data, 0x0123_4567_89AB_CDEF);

    println!("all checks passed");
}
