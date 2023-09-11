use bit_field::BitField;
use bitflags::bitflags;
use core::ptr::NonNull;
use volatile::VolatileRef;

/// Base of the redirection table.
const RDT_BASE: u8 = 0x10;

/// I/O APIC ID register
const IA_ID_REG: u8 = 0x00;

/// I/O APIC version register
const IA_VER_REG: u8 = 0x01;

/// I/O APIC arbitration register
const IA_ARB_REG: u8 = 0x02;

bitflags! {
    /// Information stored in bits 8 to 10 of the redirection table entry register for an IRQ
    /// that determines how an interrupt will be sent to the CPU.
    pub struct DeliveryMode: u8 {
        const FIXED = 1 << 0;
        const LOW_PRIORITY = 1 << 1;
        const SMI = 1 << 2;
        const NMI = 1 << 3;
        const INIT = 0b101;
        const EXTINIT = 0b111;
    }
}

/// Redirection table entry for an IRQ.
#[derive(Debug, Copy, Clone)]
pub struct RedirectionTableEntry {
    high: u32,
    low: u32,
}

/// A single I/O APIC.
pub struct IoApic {
    /// The register select register, used to access the rest of the I/O APIC's registers.
    reg_sel: VolatileRef<'static, u32>,

    /// The register which data will be returned from.
    data: VolatileRef<'static, u32>,
}

impl IoApic {
    /// Create a new I/O APIC.
    ///
    /// # Safety
    ///
    /// `base_addr` must be a valid address.
    pub unsafe fn new(base_addr: usize) -> Self {
        Self {
            // SAFETY: `base_addr` is a valid address.
            reg_sel: unsafe { VolatileRef::new(NonNull::from(&(base_addr as u32))) },
            data: unsafe { VolatileRef::new(NonNull::from(&((base_addr + 0x10) as u32))) },
        }
    }

    /// Read from the register specified by `reg`.
    pub fn read_reg(&mut self, reg: u8) -> u32 {
        self.reg_sel.as_mut_ptr().write(reg as u32);
        self.data.as_ptr().read()
    }

    /// Write `val` to the register specified by `reg`.
    pub fn write_reg(&mut self, reg: u8, val: u32) {
        self.reg_sel.as_mut_ptr().write(reg as u32);
        self.data.as_mut_ptr().write(val);
    }

    /// Return the ID of this I/O APIC.
    pub fn id(&mut self) -> u8 {
        self.read_reg(IA_ID_REG).get_bits(24..28) as u8
    }

    /// Return the version of this I/O APIC.
    pub fn version(&mut self) -> u8 {
        self.read_reg(IA_VER_REG).get_bits(0..9) as u8
    }

    /// Return the amount of IRQs this I/O APIC can handle.
    pub fn irqs(&mut self) -> u8 {
        self.read_reg(IA_VER_REG).get_bits(16..24) as u8
    }

    /// Return the arbitration ID of this I/O APIC.
    pub fn arbitration_id(&mut self) -> u8 {
        self.read_reg(IA_ARB_REG).get_bits(24..28) as u8
    }
}
