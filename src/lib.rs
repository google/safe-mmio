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
/// A `UniqueMmioPointer` may be created from a mutable reference, but this should only be used for
/// testing purposes, as references should never be constructed for real MMIO address space.
#[derive(Debug, Eq, PartialEq)]
pub struct UniqueMmioPointer<'a, T: ?Sized> {
    regs: NonNull<T>,
    phantom: PhantomData<&'a mut T>,
}

impl<T: ?Sized> UniqueMmioPointer<'_, T> {
    /// Creates a new `UniqueMmioPointer` from a non-null raw pointer.
    ///
    /// # Safety
    ///
    /// `regs` must be a properly aligned and valid pointer to some MMIO address space of type T,
    /// which is mapped as device memory and valid to read and write from any thread with volatile
    /// operations. There must not be any other aliases which are used to access the same MMIO
    /// region while this `UniqueMmioPointer` exists.
    pub unsafe fn new(regs: NonNull<T>) -> Self {
        Self {
            regs,
            phantom: PhantomData,
        }
    }

    /// Creates a new `UniqueMmioPointer` with the same lifetime as this one.
    ///
    /// This is used internally by the [`field!`] macro and shouldn't be called directly.
    ///
    /// # Safety
    ///
    /// `regs` must be a properly aligned and valid pointer to some MMIO address space of type T,
    /// within the allocation that `self` points to.
    pub unsafe fn child<U>(&mut self, regs: NonNull<U>) -> UniqueMmioPointer<U> {
        UniqueMmioPointer {
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

impl<T> UniqueMmioPointer<'_, [T]> {
    /// Returns a `UniqueMmioPointer` to an element of this slice, or `None` if the index is out of
    /// bounds.
    pub fn get(&mut self, index: usize) -> Option<UniqueMmioPointer<T>> {
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

impl<T, const LEN: usize> UniqueMmioPointer<'_, [T; LEN]> {
    /// Splits a `UniqueMmioPointer` to an array into an array of `UniqueMmioPointer`s.
    pub fn split(&mut self) -> [UniqueMmioPointer<T>; LEN] {
        array::from_fn(|i| UniqueMmioPointer {
            // SAFETY: self.regs is always unique and valid for MMIO access. We make sure the
            // pointers we split it into don't overlap, so the same applies to each of them.
            regs: NonNull::new(unsafe { &raw mut (*self.regs.as_ptr())[i] }).unwrap(),
            phantom: PhantomData,
        })
    }
}

// SAFETY: The caller of `UniqueMmioPointer::new` promises that the MMIO registers can be accessed
// from any thread.
unsafe impl<T> Send for UniqueMmioPointer<'_, T> {}

impl<'a, T: ?Sized> From<&'a mut T> for UniqueMmioPointer<'a, T> {
    fn from(r: &'a mut T) -> Self {
        Self {
            regs: r.into(),
            phantom: PhantomData,
        }
    }
}

/// Gets a `UniqueMmioPointer` to a field of a type wrapped in a `UniqueMmioPointer`.
#[macro_export]
macro_rules! field {
    ($mmio_pointer:expr, $field:ident) => {{
        // Make sure $mmio_pointer is the right type.
        let mmio_pointer: &mut $crate::UniqueMmioPointer<_> = &mut $mmio_pointer;
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
        let mut owned: UniqueMmioPointer<Foo> = UniqueMmioPointer::from(&mut foo);

        let owned_a: UniqueMmioPointer<u32> = field!(owned, a);
        assert_eq!(owned_a.read(), 1);

        let owned_b: UniqueMmioPointer<u32> = field!(owned, b);
        assert_eq!(owned_b.read(), 2);
    }

    #[test]
    fn array() {
        let mut foo = [1, 2, 3];
        let mut owned = UniqueMmioPointer::from(&mut foo);

        let parts = owned.split();
        assert_eq!(parts[0].read(), 1);
        assert_eq!(parts[1].read(), 2);
        assert_eq!(owned.split()[2].read(), 3);
    }

    #[test]
    fn slice() {
        let mut foo = [1, 2, 3];
        let mut owned = UniqueMmioPointer::from(foo.as_mut_slice());

        assert!(!owned.ptr().is_null());
        assert!(!owned.ptr_mut().is_null());

        assert!(!owned.is_empty());
        assert_eq!(owned.len(), 3);

        let first: UniqueMmioPointer<i32> = owned.get(0).unwrap();
        assert_eq!(first.read(), 1);

        let second: UniqueMmioPointer<i32> = owned.get(1).unwrap();
        assert_eq!(second.read(), 2);

        assert_eq!(owned.get(3), None);
    }
}
