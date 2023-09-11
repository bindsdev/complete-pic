//! APIC interface
//!
//! # What is the APIC?
//! The APIC (Advanced Programmable Interrupt Controller) is the successor of the legacy 8259 PIC. It allows more complex operations
//! when dealing with interrupts.
//!
//! ## How is the APIC better than the 8259 PIC?
//!
//! The APIC is more powerful than the 8259 PIC. One of the major differences is that it can be used in multiprocessor systems.
//! It can also do things that the 8259 PIC couldn't like sending interrupts between processors. Due to the APIC being more complicated
//! than the 8259 PIC, it also requires a bit more work to get it fully configured properly.
//!
//! ## Where can I read more?
//! The following links are useful to learning more about the ACPI, MADT, APIC, and interrupts:
//! - [ACPI on OSDev Wiki](https://wiki.osdev.org/ACPI)
//! - [ACPI on Wikipedia](https://en.wikipedia.org/wiki/ACPI)
//! - [APIC](https://wiki.osdev.org/APIC)
//! - [IOAPIC](https://wiki.osdev.org/IOAPIC)
//! - [APIC Timer](https://wiki.osdev.org/APIC_timer)
//! - [MADT](https://wiki.osdev.org/MADT)
//! - [Interrupts](https://wiki.osdev.org/IRQ)
//!
//! # Public API
//!
//! # Usage
//!
//! To use this interface, the `apic` Cargo feature must be enabled. It should look something like this:
//!
//! ```toml
//! [dependencies]
//! complete-pic = { version = "1.0.0", features = ["apic"] }
//! ```
//!
//! To setup the APIC properly, there are a few steps you must take:
//!
//! 1. Parse the ACPI (Advanced Configuration and Power Interface) tables, specifically the MADT (Multiple APIC Description Table). This can be done using
//!    the [acpi](https://docs.rs/acpi/latest/acpi/index.html) crate.
//! 2. Check if the legacy 8259 PICs are present. If so, you must mask all interrupts and remap the IRQs

pub mod ioapic;
pub mod lapic;
