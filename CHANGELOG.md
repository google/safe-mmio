# Changelog

## Unreleased

### Bugfixes

- Implemented `KnownLayout` for `ReadPureWrite`. This was missed accidentally before.

## 0.2.1

### New features

- Added `get` method to `UniqueMmioPointer<[T; N]>` and `SharedMmioPointer<[T; N]>`.

## 0.2.0

### Breaking changes

- Renamed `OwnedMmioPointer` to `UniqueMmioPointer`.

### New features

- Added `SharedMmioPointer` for an MMIO pointer that is not necessarily unique. Unlike a
  `UniqueMmioPointer`, a `SharedMmioPointer` can be cloned. `UniqueMmioPointer` derefs to
  `SharedMmioPointer` and can also be converted to it.
- Added `get` and `split` methods on `UniqueMmioPointer<[T]>` and `UniqueMmioPointer<[T; _]>`
  respectively, to go from a pointer to a slice or field to pointers to the individual elements.
- Added `field!` and `field_shared!` macros to go from a pointer to a struct to a pointer to an
  individual field.
- Added `write_unsafe` and `read_unsafe` methods on `UniqueMmioPointer` and `SharedMmioPointer`.
  These call `write_volatile` and `read_volatile` on most platforms, but on aarch64 are implemented
  with inline assembly instead to work around
  [a bug with how volatile writes and reads are implemented](https://github.com/rust-lang/rust/issues/131894).
- Added wrapper types `ReadOnly`, `ReadPure`, `WriteOnly`, `ReadWrite` and `ReadPureWrite` to
  indicate whether a field can safely be written or read (with or without side-effects). Added safe
  `write` and `read` methods on `UniqueMmioPointer` or `SharedMmioPointer` for these.

## 0.1.0

Initial release.
