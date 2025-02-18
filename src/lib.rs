// Copyright 2025 The safe-mmio Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

//! Types for safe MMIO device access, especially in systems with an MMU.

#![no_std]
#![deny(clippy::undocumented_unsafe_blocks)]
#![deny(unsafe_op_in_unsafe_fn)]

#[cfg(target_arch = "aarch64")]
mod aarch64_mmio;
mod physical;
#[cfg(not(target_arch = "aarch64"))]
mod volatile_mmio;

use core::{array, fmt::Debug, marker::PhantomData, ptr::NonNull};
pub use physical::PhysicalInstance;

/// A unique owned pointer to the registers of some MMIO device.
///
/// It is guaranteed to be valid and unique; no other access to the MMIO space of the device may
/// happen for the lifetime `'a`.
///
/// An `OwnedMmioPointer` may be created from a mutable reference, but this should only be used for
/// testing purposes, as references should never be constructed for real MMIO address space.
#[derive(Debug, Eq, PartialEq)]
pub struct OwnedMmioPointer<'a, T: ?Sized> {
    regs: NonNull<T>,
    phantom: PhantomData<&'a mut T>,
}

impl<T: ?Sized> OwnedMmioPointer<'_, T> {
    /// Creates a new `OwnedMmioPointer` from a non-null raw pointer.
    ///
    /// # Safety
    ///
    /// `regs` must be a properly aligned and valid pointer to some MMIO address space of type T,
    /// which is mapped as device memory and valid to read and write from any thread with volatile
    /// operations. There must not be any other aliases which are used to access the same MMIO
    /// region while this `OwnedMmioPointer` exists.
    pub unsafe fn new(regs: NonNull<T>) -> Self {
        Self {
            regs,
            phantom: PhantomData,
        }
    }

    /// Creates a new `OwnedMmioPointer` with the same lifetime as this one.
    ///
    /// This is used internally by the [`field!`] macro and shouldn't be called directly.
    ///
    /// # Safety
    ///
    /// `regs` must be a properly aligned and valid pointer to some MMIO address space of type T,
    /// within the allocation that `self` points to.
    pub unsafe fn child<U>(&mut self, regs: NonNull<U>) -> OwnedMmioPointer<U> {
        OwnedMmioPointer {
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

impl<T> OwnedMmioPointer<'_, [T]> {
    /// Returns an `OwnedMmioPointer` to an element of this slice, or `None` if the index is out of
    /// bounds.
    pub fn get(&mut self, index: usize) -> Option<OwnedMmioPointer<T>> {
        if index >= self.len() {
            return None;
        }
        // SAFETY: self.regs is always unique and valid for MMIO access.
        let regs = NonNull::new(unsafe { &raw mut (*self.regs.as_ptr())[index] }).unwrap();
        // SAFETY: We created regs from the raw slice in self.regs, so it must also be valid, unique
        // and within the allocation of self.regs.
        Some(unsafe { self.child(regs) })
    }

    /// Returns the length of the slice.
    pub const fn len(&self) -> usize {
        self.regs.len()
    }

    /// Returns whether the slice is empty.
    pub const fn is_empty(&self) -> bool {
        self.regs.is_empty()
    }
}

impl<T, const LEN: usize> OwnedMmioPointer<'_, [T; LEN]> {
    /// Splits an `OwnedMmioPointer` to an array into an array of `OwnedMmioPointer`s.
    pub fn split(&mut self) -> [OwnedMmioPointer<T>; LEN] {
        array::from_fn(|i| OwnedMmioPointer {
            // SAFETY: self.regs is always unique and valid for MMIO access. We make sure the
            // pointers we split it into don't overlap, so the same applies to each of them.
            regs: NonNull::new(unsafe { &raw mut (*self.regs.as_ptr())[i] }).unwrap(),
            phantom: PhantomData,
        })
    }
}

// SAFETY: The caller of `OwnedMmioPointer::new` promises that the MMIO registers can be accessed
// from any thread.
unsafe impl<T> Send for OwnedMmioPointer<'_, T> {}

impl<'a, T: ?Sized> From<&'a mut T> for OwnedMmioPointer<'a, T> {
    fn from(r: &'a mut T) -> Self {
        Self {
            regs: r.into(),
            phantom: PhantomData,
        }
    }
}

/// Gets an `OwnedMmioPointer` to a field of a type wrapped in an `OwnedMmioPointer`.
#[macro_export]
macro_rules! field {
    ($mmio_pointer:expr, $field:ident) => {{
        // Make sure $mmio_pointer is the right type.
        let mmio_pointer: &mut $crate::OwnedMmioPointer<_> = &mut $mmio_pointer;
        // SAFETY: ptr_mut is guaranteed to return a valid pointer for MMIO, so the pointer to the
        // field must also be valid. MmioPointer::child gives it the same lifetime as the original
        // pointer.
        unsafe {
            let child_pointer =
                core::ptr::NonNull::new(&raw mut (*mmio_pointer.ptr_mut()).$field).unwrap();
            mmio_pointer.child(child_pointer)
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields() {
        struct Foo {
            a: u32,
            b: u32,
        }

        let mut foo = Foo { a: 1, b: 2 };
        let mut owned: OwnedMmioPointer<Foo> = OwnedMmioPointer::from(&mut foo);

        let owned_a: OwnedMmioPointer<u32> = field!(owned, a);
        assert_eq!(owned_a.read(), 1);

        let owned_b: OwnedMmioPointer<u32> = field!(owned, b);
        assert_eq!(owned_b.read(), 2);
    }

    #[test]
    fn array() {
        let mut foo = [1, 2, 3];
        let mut owned = OwnedMmioPointer::from(&mut foo);

        let parts = owned.split();
        assert_eq!(parts[0].read(), 1);
        assert_eq!(parts[1].read(), 2);
        assert_eq!(owned.split()[2].read(), 3);
    }

    #[test]
    fn slice() {
        let mut foo = [1, 2, 3];
        let mut owned = OwnedMmioPointer::from(foo.as_mut_slice());

        assert!(!owned.ptr().is_null());
        assert!(!owned.ptr_mut().is_null());

        assert!(!owned.is_empty());
        assert_eq!(owned.len(), 3);

        let first: OwnedMmioPointer<i32> = owned.get(0).unwrap();
        assert_eq!(first.read(), 1);

        let second: OwnedMmioPointer<i32> = owned.get(1).unwrap();
        assert_eq!(second.read(), 2);

        assert_eq!(owned.get(3), None);
    }
}
