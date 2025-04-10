use crate::{SharedMmioPointer, UniqueMmioPointer, mmio::{self, ReadVolatile, WriteVolatile}};

debug_assertion!(!mmio::is_volatile(context)?);

crate::SharedMmioPointer<'_, T> {
    /// Performs an MMIO read of the entire `T`.
    ///
    /// # Panics
    ///
    /// # Safety
    ///
    /// # Example
    pub fn read_unsafe(&self) -> T {
        ReadVolatile::<T>::new(&self.regs).read()
    }
}

crate::UniqueMmioPointer<'_, T> {
    /// Performs an MMIO read of the entire `T`.
    ///
    /// # Panics
    ///
    /// # Safety
    ///
    /// # Example
    pub fn read_unsafe(&mut self) -> T {
        ReadVolatile::<T>::new(&self.regs).read()
    }

    pub fn write_unsafe(&mut self, value: T) {
        WriteVolatile::<T>::new(&self.regs).write(value)
    }
}

// TODO(marcan): turn the ReadVolatile/WriteVolatile methods into separate
// functions so they can be inlined by the compiler.