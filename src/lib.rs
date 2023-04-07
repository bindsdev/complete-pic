//! A complete interface for both the legacy [8259 PIC](https://wiki.osdev.org/PIC) and the newer
//! [APIC](https://wiki.osdev.org/APIC). More specific documentation can be found by reading the documentation of the
//! modules designated for the [8259 PIC](crate::pic8259) and APIC or by visiting the OSDev wiki's pages on them, which are hyperlinked above.
//!
//! # Usage
//! To use this crate, add it to your `Cargo.toml` file:
//! ```toml
//! [dependencies]
//! complete_pic = "0.1.1"
//! ```   
//!
//! ## Crate Features
//! - `8259pic` - Enable interface for the legacy 8259 PIC (not on by default)
//! - `apic` - Enable interface for the newer APIC (enabled by default; crate assumes you want to use the more modern variant)

#![no_std]
#![cfg(any(target_arch = "x86", target_arch = "x86_64"))]

#[cfg(feature = "8259pic")]
pub mod pic8259;
