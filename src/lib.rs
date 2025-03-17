// Copyright 2025 The safe-mmio Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

//! Types for safe MMIO device access, especially in systems with an MMU.

#![no_std]
#![deny(clippy::undocumented_unsafe_blocks)]
#![deny(unsafe_op_in_unsafe_fn)]

#[cfg(target_arch = "aarch64")]
mod aarch64_mmio;
pub mod fields;
mod physical;
#[cfg(not(target_arch = "aarch64"))]
mod volatile_mmio;

use crate::fields::{ReadOnly, ReadPure, ReadPureWrite, ReadWrite, WriteOnly};
use core::{array, fmt::Debug, marker::PhantomData, ops::Deref, ptr, ptr::NonNull};
pub use physical::PhysicalInstance;
use zerocopy::{FromBytes, Immutable, IntoBytes};

/// A unique owned pointer to the registers of some MMIO device.
///
/// It is guaranteed to be valid and unique; no other access to the MMIO space of the device may
/// happen for the lifetime `'a`.
///
/// A `UniqueMmioPointer` may be created from a mutable reference, but this should only be used for
/// testing purposes, as references should never be constructed for real MMIO address space.
pub struct UniqueMmioPointer<'a, T: ?Sized>(SharedMmioPointer<'a, T>);

// Implement Debug, Eq and PartialEq manually rather than deriving to avoid an unneccessary bound on
// T.

impl<T: ?Sized> Debug for UniqueMmioPointer<'_, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("UniqueMmioPointer")
            .field(&self.0.regs)
            .finish()
    }
}

impl<T: ?Sized> PartialEq for UniqueMmioPointer<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: ?Sized> Eq for UniqueMmioPointer<'_, T> {}

impl<T: ?Sized> UniqueMmioPointer<'_, T> {
    /// Creates a new `UniqueMmioPointer` from a non-null raw pointer.
    ///
    /// # Safety
    ///
    /// `regs` must be a properly aligned and valid pointer to some MMIO address space of type T,
    /// which is mapped as device memory and valid to read and write from any thread with volatile
    /// operations. There must not be any other aliases which are used to access the same MMIO
    /// region while this `UniqueMmioPointer` exists.
    ///
    /// If `T` contains any fields wrapped in [`ReadOnly`], [`WriteOnly`] or [`ReadWrite`] then they
    /// must indeed be safe to perform MMIO reads or writes on.
    pub unsafe fn new(regs: NonNull<T>) -> Self {
        Self(SharedMmioPointer {
            regs,
            phantom: PhantomData,
        })
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
        UniqueMmioPointer(SharedMmioPointer {
            regs,
            phantom: PhantomData,
        })
    }

    /// Returns a raw mut pointer to the MMIO registers.
    pub fn ptr_mut(&mut self) -> *mut T {
        self.0.regs.as_ptr()
    }

    /// Returns a `NonNull<T>` pointer to the MMIO registers.
    pub fn ptr_nonnull(&mut self) -> NonNull<T> {
        self.0.regs
    }
}

impl<T: FromBytes + IntoBytes> UniqueMmioPointer<'_, ReadWrite<T>> {
    /// Performs an MMIO read of the entire `T`.
    pub fn read(&mut self) -> T {
        // SAFETY: self.regs is always a valid and unique pointer to MMIO address space, and `T`
        // being wrapped in `ReadWrite` implies that it is safe to read.
        unsafe { self.read_unsafe().0 }
    }
}

impl<T: Immutable + IntoBytes> UniqueMmioPointer<'_, ReadWrite<T>> {
    /// Performs an MMIO write of the entire `T`.
    pub fn write(&mut self, value: T) {
        // SAFETY: self.regs is always a valid and unique pointer to MMIO address space, and `T`
        // being wrapped in `ReadWrite` implies that it is safe to write.
        unsafe {
            self.write_unsafe(ReadWrite(value));
        }
    }
}

impl<T: Immutable + IntoBytes> UniqueMmioPointer<'_, ReadPureWrite<T>> {
    /// Performs an MMIO write of the entire `T`.
    pub fn write(&mut self, value: T) {
        // SAFETY: self.regs is always a valid and unique pointer to MMIO address space, and `T`
        // being wrapped in `ReadPureWrite` implies that it is safe to write.
        unsafe {
            self.write_unsafe(ReadPureWrite(value));
        }
    }
}

impl<T: FromBytes + IntoBytes> UniqueMmioPointer<'_, ReadOnly<T>> {
    /// Performs an MMIO read of the entire `T`.
    pub fn read(&mut self) -> T {
        // SAFETY: self.regs is always a valid and unique pointer to MMIO address space, and `T`
        // being wrapped in `ReadOnly` implies that it is safe to read.
        unsafe { self.read_unsafe().0 }
    }
}

impl<T: Immutable + IntoBytes> UniqueMmioPointer<'_, WriteOnly<T>> {
    /// Performs an MMIO write of the entire `T`.
    pub fn write(&mut self, value: T) {
        // SAFETY: self.regs is always a valid and unique pointer to MMIO address space, and `T`
        // being wrapped in `WriteOnly` implies that it is safe to write.
        unsafe {
            self.write_unsafe(WriteOnly(value));
        }
    }
}

impl<T> UniqueMmioPointer<'_, [T]> {
    /// Returns a `UniqueMmioPointer` to an element of this slice, or `None` if the index is out of
    /// bounds.
    pub fn get(&mut self, index: usize) -> Option<UniqueMmioPointer<T>> {
        if index >= self.len() {
            return None;
        }
        // SAFETY: self.ptr_mut() is guaranteed to return a pointer that is valid for MMIO and
        // unique, as promised by the caller of `UniqueMmioPointer::new`.
        let regs = NonNull::new(unsafe { &raw mut (*self.ptr_mut())[index] }).unwrap();
        // SAFETY: We created regs from the raw slice in self.regs, so it must also be valid, unique
        // and within the allocation of self.regs.
        Some(unsafe { self.child(regs) })
    }
}

impl<T, const LEN: usize> UniqueMmioPointer<'_, [T; LEN]> {
    /// Splits a `UniqueMmioPointer` to an array into an array of `UniqueMmioPointer`s.
    pub fn split(&mut self) -> [UniqueMmioPointer<T>; LEN] {
        array::from_fn(|i| {
            UniqueMmioPointer(SharedMmioPointer {
                // SAFETY: self.regs is always unique and valid for MMIO access. We make sure the
                // pointers we split it into don't overlap, so the same applies to each of them.
                regs: NonNull::new(unsafe { &raw mut (*self.ptr_mut())[i] }).unwrap(),
                phantom: PhantomData,
            })
        })
    }

    /// Returns a `UniqueMmioPointer` to an element of this array, or `None` if the index is out of
    /// bounds.
    ///
    /// # Example
    ///
    /// ```
    /// use safe_mmio::{UniqueMmioPointer, fields::ReadWrite};
    ///
    /// let mut slice: UniqueMmioPointer<[ReadWrite<u32>; 3]>;
    /// # let mut fake = [ReadWrite(1), ReadWrite(2), ReadWrite(3)];
    /// # slice = UniqueMmioPointer::from(&mut fake);
    /// let mut element = slice.get(1).unwrap();
    /// element.write(42);
    /// ```
    pub fn get(&mut self, index: usize) -> Option<UniqueMmioPointer<T>> {
        if index >= LEN {
            return None;
        }
        // SAFETY: self.ptr_mut() is guaranteed to return a pointer that is valid for MMIO and
        // unique, as promised by the caller of `UniqueMmioPointer::new`.
        let regs = NonNull::new(unsafe { &raw mut (*self.ptr_mut())[index] }).unwrap();
        // SAFETY: We created regs from the raw array in self.regs, so it must also be valid, unique
        // and within the allocation of self.regs.
        Some(unsafe { self.child(regs) })
    }
}

impl<'a, T: ?Sized> From<&'a mut T> for UniqueMmioPointer<'a, T> {
    fn from(r: &'a mut T) -> Self {
        Self(SharedMmioPointer {
            regs: r.into(),
            phantom: PhantomData,
        })
    }
}

impl<'a, T: ?Sized> Deref for UniqueMmioPointer<'a, T> {
    type Target = SharedMmioPointer<'a, T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A shared pointer to the registers of some MMIO device.
///
/// It is guaranteed to be valid but unlike [`UniqueMmioPointer`] may not be unique.
pub struct SharedMmioPointer<'a, T: ?Sized> {
    regs: NonNull<T>,
    phantom: PhantomData<&'a T>,
}

// Implement Debug, Eq and PartialEq manually rather than deriving to avoid an unneccessary bound on
// T.

impl<T: ?Sized> Debug for SharedMmioPointer<'_, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("SharedMmioPointer")
            .field(&self.regs)
            .finish()
    }
}

impl<T: ?Sized> PartialEq for SharedMmioPointer<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self.regs.as_ptr(), other.regs.as_ptr())
    }
}

impl<T: ?Sized> Eq for SharedMmioPointer<'_, T> {}

impl<T: ?Sized> Clone for SharedMmioPointer<'_, T> {
    fn clone(&self) -> Self {
        Self {
            regs: self.regs.clone(),
            phantom: self.phantom.clone(),
        }
    }
}

impl<T: ?Sized> SharedMmioPointer<'_, T> {
    /// Creates a new `SharedMmioPointer` with the same lifetime as this one.
    ///
    /// This is used internally by the [`field_shared!`] macro and shouldn't be called directly.
    ///
    /// # Safety
    ///
    /// `regs` must be a properly aligned and valid pointer to some MMIO address space of type T,
    /// within the allocation that `self` points to.
    pub unsafe fn child<U>(&self, regs: NonNull<U>) -> SharedMmioPointer<U> {
        SharedMmioPointer {
            regs,
            phantom: PhantomData,
        }
    }

    /// Returns a raw const pointer to the MMIO registers.
    pub fn ptr(&self) -> *const T {
        self.regs.as_ptr()
    }
}

// SAFETY: A `SharedMmioPointer` always originates either from a reference or from a
// `UniqueMmioPointer`. The caller of `UniqueMmioPointer::new` promises that the MMIO registers can
// be accessed from any thread.
unsafe impl<T: ?Sized + Send + Sync> Send for SharedMmioPointer<'_, T> {}

impl<'a, T: ?Sized> From<&'a T> for SharedMmioPointer<'a, T> {
    fn from(r: &'a T) -> Self {
        Self {
            regs: r.into(),
            phantom: PhantomData,
        }
    }
}

impl<'a, T: ?Sized> From<UniqueMmioPointer<'a, T>> for SharedMmioPointer<'a, T> {
    fn from(unique: UniqueMmioPointer<'a, T>) -> Self {
        unique.0
    }
}

impl<T: FromBytes + IntoBytes> SharedMmioPointer<'_, ReadPure<T>> {
    /// Performs an MMIO read of the entire `T`.
    pub fn read(&self) -> T {
        // SAFETY: self.regs is always a valid and unique pointer to MMIO address space, and `T`
        // being wrapped in `ReadPure` implies that it is safe to read from a shared reference
        // because doing so has no side-effects.
        unsafe { self.read_unsafe().0 }
    }
}

impl<T: FromBytes + IntoBytes> SharedMmioPointer<'_, ReadPureWrite<T>> {
    /// Performs an MMIO read of the entire `T`.
    pub fn read(&self) -> T {
        // SAFETY: self.regs is always a valid pointer to MMIO address space, and `T`
        // being wrapped in `ReadPureWrite` implies that it is safe to read from a shared reference
        // because doing so has no side-effects.
        unsafe { self.read_unsafe().0 }
    }
}

impl<T> SharedMmioPointer<'_, [T]> {
    /// Returns a `SharedMmioPointer` to an element of this slice, or `None` if the index is out of
    /// bounds.
    pub fn get(&self, index: usize) -> Option<SharedMmioPointer<T>> {
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

impl<T, const LEN: usize> SharedMmioPointer<'_, [T; LEN]> {
    /// Splits a `SharedMmioPointer` to an array into an array of `SharedMmioPointer`s.
    pub fn split(&self) -> [SharedMmioPointer<T>; LEN] {
        array::from_fn(|i| SharedMmioPointer {
            // SAFETY: self.regs is always unique and valid for MMIO access. We make sure the
            // pointers we split it into don't overlap, so the same applies to each of them.
            regs: NonNull::new(unsafe { &raw mut (*self.regs.as_ptr())[i] }).unwrap(),
            phantom: PhantomData,
        })
    }

    /// Returns a `SharedMmioPointer` to an element of this array, or `None` if the index is out of
    /// bounds.
    pub fn get(&self, index: usize) -> Option<SharedMmioPointer<T>> {
        if index >= LEN {
            return None;
        }
        // SAFETY: self.regs is always unique and valid for MMIO access.
        let regs = NonNull::new(unsafe { &raw mut (*self.regs.as_ptr())[index] }).unwrap();
        // SAFETY: We created regs from the raw array in self.regs, so it must also be valid, unique
        // and within the allocation of self.regs.
        Some(unsafe { self.child(regs) })
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

/// Gets a `SharedMmioPointer` to a field of a type wrapped in a `SharedMmioPointer`.
#[macro_export]
macro_rules! field_shared {
    ($mmio_pointer:expr, $field:ident) => {{
        // Make sure $mmio_pointer is the right type.
        let mmio_pointer: &$crate::SharedMmioPointer<_> = &$mmio_pointer;
        // SAFETY: ptr_mut is guaranteed to return a valid pointer for MMIO, so the pointer to the
        // field must also be valid. MmioPointer::child gives it the same lifetime as the original
        // pointer.
        unsafe {
            let child_pointer =
                core::ptr::NonNull::new((&raw const (*mmio_pointer.ptr()).$field).cast_mut())
                    .unwrap();
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
            a: ReadWrite<u32>,
            b: ReadOnly<u32>,
            c: ReadPure<u32>,
        }

        let mut foo = Foo {
            a: ReadWrite(1),
            b: ReadOnly(2),
            c: ReadPure(3),
        };
        let mut owned: UniqueMmioPointer<Foo> = UniqueMmioPointer::from(&mut foo);

        let mut owned_a: UniqueMmioPointer<ReadWrite<u32>> = field!(owned, a);
        assert_eq!(owned_a.read(), 1);
        owned_a.write(42);
        assert_eq!(owned_a.read(), 42);
        field!(owned, a).write(44);
        assert_eq!(field!(owned, a).read(), 44);

        let mut owned_b: UniqueMmioPointer<ReadOnly<u32>> = field!(owned, b);
        assert_eq!(owned_b.read(), 2);

        let owned_c: UniqueMmioPointer<ReadPure<u32>> = field!(owned, c);
        assert_eq!(owned_c.read(), 3);
        assert_eq!(field!(owned, c).read(), 3);
    }

    #[test]
    fn shared_fields() {
        struct Foo {
            a: ReadPureWrite<u32>,
            b: ReadPure<u32>,
        }

        let foo = Foo {
            a: ReadPureWrite(1),
            b: ReadPure(2),
        };
        let shared: SharedMmioPointer<Foo> = SharedMmioPointer::from(&foo);

        let shared_a: SharedMmioPointer<ReadPureWrite<u32>> = field_shared!(shared, a);
        assert_eq!(shared_a.read(), 1);
        assert_eq!(field_shared!(shared, a).read(), 1);

        let shared_b: SharedMmioPointer<ReadPure<u32>> = field_shared!(shared, b);
        assert_eq!(shared_b.read(), 2);
    }

    #[test]
    fn shared_from_unique() {
        struct Foo {
            a: ReadPureWrite<u32>,
            b: ReadPure<u32>,
        }

        let mut foo = Foo {
            a: ReadPureWrite(1),
            b: ReadPure(2),
        };
        let unique: UniqueMmioPointer<Foo> = UniqueMmioPointer::from(&mut foo);

        let shared_a: SharedMmioPointer<ReadPureWrite<u32>> = field_shared!(unique, a);
        assert_eq!(shared_a.read(), 1);

        let shared_b: SharedMmioPointer<ReadPure<u32>> = field_shared!(unique, b);
        assert_eq!(shared_b.read(), 2);
    }

    #[test]
    fn restricted_fields() {
        struct Foo {
            r: ReadOnly<u32>,
            w: WriteOnly<u32>,
            u: u32,
        }

        let mut foo = Foo {
            r: ReadOnly(1),
            w: WriteOnly(2),
            u: 3,
        };
        let mut owned: UniqueMmioPointer<Foo> = UniqueMmioPointer::from(&mut foo);

        let mut owned_r: UniqueMmioPointer<ReadOnly<u32>> = field!(owned, r);
        assert_eq!(owned_r.read(), 1);

        let mut owned_w: UniqueMmioPointer<WriteOnly<u32>> = field!(owned, w);
        owned_w.write(42);

        let mut owned_u: UniqueMmioPointer<u32> = field!(owned, u);
        // SAFETY: 'u' is safe to read or write because it's just a fake.
        unsafe {
            assert_eq!(owned_u.read_unsafe(), 3);
            owned_u.write_unsafe(42);
            assert_eq!(owned_u.read_unsafe(), 42);
        }
    }

    #[test]
    fn array() {
        let mut foo = [ReadWrite(1), ReadWrite(2), ReadWrite(3)];
        let mut owned = UniqueMmioPointer::from(&mut foo);

        let mut parts = owned.split();
        assert_eq!(parts[0].read(), 1);
        assert_eq!(parts[1].read(), 2);
        assert_eq!(owned.split()[2].read(), 3);
    }

    #[test]
    fn array_shared() {
        let foo = [ReadPure(1), ReadPure(2), ReadPure(3)];
        let shared = SharedMmioPointer::from(&foo);

        let parts = shared.split();
        assert_eq!(parts[0].read(), 1);
        assert_eq!(parts[1].read(), 2);
        assert_eq!(shared.split()[2].read(), 3);
    }

    #[test]
    fn slice() {
        let mut foo = [ReadWrite(1), ReadWrite(2), ReadWrite(3)];
        let mut owned = UniqueMmioPointer::from(foo.as_mut_slice());

        assert!(!owned.ptr().is_null());
        assert!(!owned.ptr_mut().is_null());

        assert!(!owned.is_empty());
        assert_eq!(owned.len(), 3);

        let mut first: UniqueMmioPointer<ReadWrite<i32>> = owned.get(0).unwrap();
        assert_eq!(first.read(), 1);

        let mut second: UniqueMmioPointer<ReadWrite<i32>> = owned.get(1).unwrap();
        assert_eq!(second.read(), 2);

        assert!(owned.get(3).is_none());
    }

    #[test]
    fn slice_shared() {
        let foo = [ReadPure(1), ReadPure(2), ReadPure(3)];
        let shared = SharedMmioPointer::from(foo.as_slice());

        assert!(!shared.ptr().is_null());

        assert!(!shared.is_empty());
        assert_eq!(shared.len(), 3);

        let first: SharedMmioPointer<ReadPure<i32>> = shared.get(0).unwrap();
        assert_eq!(first.read(), 1);

        let second: SharedMmioPointer<ReadPure<i32>> = shared.get(1).unwrap();
        assert_eq!(second.read(), 2);

        assert!(shared.get(3).is_none());
    }
}
