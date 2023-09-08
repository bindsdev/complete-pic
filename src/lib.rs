//! A complete interface for both the legacy 8259 PIC and the newer APIC. More specific documentation can be found by reading the documentation of the
//! modules designated for the [8259 PIC](crate::pic8259) and APIC.
//!
//! # Usage
//! To use this crate, add it to your `Cargo.toml` file:
//! ```toml
//! [dependencies]
//! complete_pic = { version = "1.0.0", features = [...] }
//! ```   
//!
//! ## Crate Features
//! - `8259pic` - Enable interface for the legacy 8259 PIC
//! - `apic` - Enable interface for the newer APIC

#![no_std]

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
compile_error!("This crate will only work on x86 platforms");

#[cfg(feature = "8259pic")]
pub mod pic8259;

#[cfg(feature = "apic")]
pub mod apic;
