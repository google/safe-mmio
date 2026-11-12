// Copyright 2025 The safe-mmio Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

//! Wrapper types for MMIO fields.

use core::mem::MaybeUninit;
use zerocopy::{FromBytes, FromZeros, Immutable, IntoBytes, KnownLayout};

/// Wrapper for a field which may safely be read but not written. Reading may cause side-effects,
/// changing the state of the device in some way.
#[derive(Clone, Debug, Default, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(transparent)]
pub struct ReadOnly<T>(pub T);

/// Wrapper for a field which may safely be read with no side-effects but not written.
#[derive(Clone, Debug, Default, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(transparent)]
pub struct ReadPure<T>(pub T);

/// Wrapper for a field which may safely be written but not read.
#[derive(Clone, Debug, Default, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(transparent)]
pub struct WriteOnly<T>(pub T);

/// Wrapper for a field which may safely be written and read. Reading may cause side-effects,
/// changing the state of the device in some way.
#[derive(Clone, Debug, Default, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(transparent)]
pub struct ReadWrite<T>(pub T);

/// Wrapper for a field which may safely be written (with side-effects) and read with no
/// side-effects.
#[derive(Clone, Debug, Default, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(transparent)]
pub struct ReadPureWrite<T>(pub T);

/// Wrapper for a field which must neither be read nor written, such as an inaccessible hole or
/// padding within a contiguous MMIO region.
#[derive(FromZeros, Immutable, KnownLayout)]
#[repr(transparent)]
pub struct Inaccessible<T>(pub(crate) MaybeUninit<T>);
