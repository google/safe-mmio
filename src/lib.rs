// Copyright 2025 The safe-mmio Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

//! Types for safe MMIO device access, especially in systems with an MMU.

#![no_std]
#![deny(clippy::undocumented_unsafe_blocks)]
#![deny(unsafe_op_in_unsafe_fn)]

use core::{
    fmt::{self, Debug, Formatter},
    marker::PhantomData,
    ptr::NonNull,
};

/// A unique owned pointer to the registers of some MMIO device.
///
/// It is guaranteed to be valid and unique; no other access to the MMIO space of the device may
/// happen for the lifetime `'a`.
#[derive(Debug, Eq, PartialEq)]
pub struct OwnedMmioPointer<'a, T> {
    regs: NonNull<T>,
    phantom: PhantomData<&'a mut T>,
}

impl<T> OwnedMmioPointer<'_, T> {
    /// Creates a new `OwnedMmioPointer` from a non-null raw pointer.
    ///
    /// # Safety
    ///
    /// `ptr` must be a properly aligned and valid pointer to some MMIO address space of type T,
    /// which is mapped as device memory and valid to read and write from any thread with volatile
    /// operations. There must not be any other aliases which are used to access the same MMIO
    /// region while this `OwnedMmioPointer` exists.
    pub unsafe fn new(regs: NonNull<T>) -> Self {
        Self {
            regs,
            phantom: PhantomData,
        }
    }

    /// Returns a raw const pointer to the MMIO registers.
    pub fn ptr(&self) -> *const T {
        self.regs.as_ptr()
    }

    /// Returns a raw mut pointer to the MMIO registers.
    pub fn ptr_mut(&mut self) -> *mut T {
        self.regs.as_ptr()
    }
}

// SAFETY: The caller of `OwnedMmioPointer::new` promises that the MMIO registers can be accessed
// from any thread.
unsafe impl<T> Send for OwnedMmioPointer<'_, T> {}

impl<'a, T> From<&'a mut T> for OwnedMmioPointer<'a, T> {
    fn from(r: &'a mut T) -> Self {
        Self {
            regs: r.into(),
            phantom: PhantomData,
        }
    }
}

/// The physical instance of some device's MMIO space.
pub struct PhysicalInstance<T> {
    pa: usize,
    _phantom: PhantomData<T>,
}

impl<T> Debug for PhysicalInstance<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("PhysicalInstance")
            .field("pa", &self.pa)
            .field("size", &size_of::<T>())
            .finish()
    }
}

impl<T> PhysicalInstance<T> {
    /// # Safety
    ///
    /// This must refer to the physical address of a real set of device registers of type `T`, and
    /// there must only ever be a single `PhysicalInstance` created for those device registers.
    pub unsafe fn new(pa: usize) -> Self {
        Self {
            pa,
            _phantom: PhantomData,
        }
    }

    /// Returns the physical base address of the device's registers.
    pub fn pa(&self) -> usize {
        self.pa
    }
}
