// Copyright 2025 The safe-mmio Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

use crate::OwnedMmioPointer;

impl<T> OwnedMmioPointer<'_, T> {
    /// Performs an MMIO read of the entire `T`.
    pub fn read(&self) -> T {
        // SAFETY: self.regs is always a valid and unique pointer to MMIO address space.
        unsafe { self.regs.read_volatile() }
    }

    /// Performs an MMIO write of the entire `T`.
    pub fn write(&mut self, value: T) {
        // SAFETY: self.regs is always a valid and unique pointer to MMIO address space.
        unsafe {
            self.regs.write_volatile(value);
        }
    }
}
