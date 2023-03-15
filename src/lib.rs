//! This crate provides a complete interface for both the legacy [8259 PIC](https://wiki.osdev.org/PIC) and the newer
//! [APIC](https://wiki.osdev.org/APIC). More specific documentation can be found by reading the documentation of the
//! modules designated for the 8259 PIC and APIC or by visiting the OSDev wiki's pages on them, which are hyperlinked above.

#![no_std]
#![cfg(any(target_arch = "x86", target_arch = "x86_64"))]
