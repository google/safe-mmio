// Copyright 2025 The safe-mmio Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

macro_rules! asm_mmio {
    ($t:ty, $read_assembly:literal, $write_assembly:literal) => {
        impl $crate::OwnedMmioPointer<'_, $t> {
            #[doc = "Performs an MMIO read of the "]
            #[doc = stringify!($t)]
            #[doc = "."]
            pub fn read(&self) -> $t {
                let value;
                unsafe {
                    core::arch::asm!(
                        $read_assembly,
                        value = out(reg) value,
                        ptr = in(reg) self.regs.as_ptr(),
                    );
                }
                value
            }

            #[doc = "Performs an MMIO write of the "]
            #[doc = stringify!($t)]
            #[doc = "."]
            pub fn write(&mut self, value: $t) {
                unsafe {
                    core::arch::asm!(
                        $write_assembly,
                        value = in(reg) value,
                        ptr = in(reg) self.regs.as_ptr(),
                    );
                }
            }
        }
    };
}

asm_mmio!(u8, "ldrb {value:w}, [{ptr}]", "strb {value:w}, [{ptr}]");
asm_mmio!(u16, "ldrh {value:w}, [{ptr}]", "strh {value:w}, [{ptr}]");
asm_mmio!(u32, "ldr {value:w}, [{ptr}]", "str {value:w}, [{ptr}]");
asm_mmio!(u64, "ldr {value:x}, [{ptr}]", "str {value:x}, [{ptr}]");
