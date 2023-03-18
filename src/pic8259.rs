//! 8269 PIC interface
//!
//! # What is the 8259 PIC?
//! The 8259 PIC (or Programmable Interrupt Controller) is a crucial component of the x86 architecture.
//! It led to the x86 architecture becoming interrupt-driven. Its purpose is to manage hardware interrupts
//! by sending them to the appropriate system interrupt. This allows the system to repond to the needs of devices
//! without losing time. In modern systems, the 8259 PIC has been replaced by the APIC (Advanced Programmable Interrupt Controller),
//! which is usable with multiple cores/processors.
//!
//! ## What exactly does the 8259 PIC do?
//! The 8259 PIC controls the interrupt mechanism of the CPU. It does this by feeding multiple interrupt requests, in order, to the
//! processor. A hardware interrupt will send a pulse along its interrupt line to the 8259 PIC. The 8259 PIC will then
//! translate the IRQ (Interrupt Request)/Hardware Interrupt into a system interrupt. It will then send a message to the CPU, interrupting
//! whatever task it was doing. The OS kernel should handle these IRQs and perform the necessary procedure (like polling the keyboard for a scancode)
//! or alert a userspace program of an interrupt by sending a message to a driver.
//!
//! ## What would be different if the 8259 PIC didn't exist?
//! Without the 8259 PIC, you would have to manually poll devices in the system to figure out if they want to do anything. You
//! would then waste time trying to go to these devices to figure out what they want to do. The 8259 PIC makes it easy by allowing the devices
//! to present themselves to you when they are ready to carry out an event.
//!
//! ## How does the 8259 PIC work?
//! Modern systems contain 2 8259 PICs, each with 8 inputs. One is called the "master" and the other is called the "slave". If any input on the PIC is raised,
//! it sets a bit interanlly that signals that the input needs servicing. Next, it checks if that channel is masked or not, and whether an interrupt is already pending.
//! If the channel is unmasked and no interrupt is pending, the PIC raises the interrupt line. The slave then feeds the IRQ number to the master and the master connects to the interrupt line.
//! After the processor accepts the interrupt, the master checks which of the PICs is reponsible for answering. It then eithr feeds the interrupt number to the processor or asks the slave to.
//! The PIC that answers, whether it be the master or slave, looks for the "vector offet" and adds it to the input line to compute the interrupt number. The processor
//! then acts on that interrupt address.
//!
//! ## Where can I read more?
//! The following links are useful to learning more about the 8259 PIC and interrupts:
//! - [8259 PIC on OSDev Wiki](https://wiki.osdev.org/PIC) (the page followed to write this module and where most of the documentation above is from)
//! - [8259 PIC on Wikipedia](https://en.wikipedia.org/wiki/Intel_8259)
//! - [Interrupts](https://wiki.osdev.org/IRQ)
//!
//! # Public API
//!
//! This module is based of the design of the already existing [pic8259](https://github.com/rust-osdev/pic8259) crate. The public functions are marked as `unsafe` because it is
//! very easy to cause undefined behavior by passing incorrect values that misconfigure the 8259 PIC or using the 8259 PIC incorrectly.
//!
//! # Usage
//!
//! Before utilizing this module, it is recommended that you wrap the `ChainedPics` struct in a `Mutex` to get safe mutable access to it. This can be done by using the `spin` crate.
//! Make sure to add the `spin` crate to your `Cargo.toml` under `[dependencies]`.

use x86_64::instructions::port::Port;

/// The command I/O port of the master PIC.
const MASTER_CMD: u8 = 0x20;

/// The data I/O port of the master PIC.
const MASTER_DATA: u8 = 0x21;

/// The command I/O port of the slave PIC.
const SLAVE_CMD: u8 = 0xA0;

/// The data I/O port of the slave PIC.
const SLAVE_DATA: u8 = 0xA1;

/// PIC initialization command.
const PIC_INIT: u8 = 0x11;

/// PIC End of Interrupt command.
const PIC_EIO: u8 = 0x20;

/// An individual PIC chip.
struct Pic {
    /// The vector offset of the PIC chip.
    offset: u8,

    /// The PIC chip's command I/O port.
    command: Port<u8>,

    /// The PIC chip's data I/O port.
    data: Port<u8>,
}

impl Pic {
    /// Create an instance of a PIC chip by providing its
    /// offset and the command and data I/O port addresses.
    fn new(offset: u8, command: u8, data: u8) -> Self {
        Self {
            offset,
            command: Port::new(command),
            data: Port::new(data),
        }
    }

    /// Check if this PIC is in charge of handling the IRQ specified by the given ID
    /// (each PIC handles 8 interrupts).
    const fn handles_interrupt(&self, irq_id: u8) -> bool {
        self.offset <= irq_id && irq_id < self.offset + 8
    }

    /// Signal that an IRQ has been handled and that the PIC is ready for more IRQs
    unsafe fn end_of_interrupt(&mut self) {
        self.command.write(PIC_EIO);
    }

    /// Read the interrupt mask of this PIC. When no command is issued, we can access the PIC's
    /// interrupt mask via its data I/O port.
    unsafe fn read_interrupt_mask(&mut self) -> u8 {
        self.data.read()
    }

    /// Write to the interrupt mask of this PIC. When no command is issued, we can access the PIC's
    /// interrupt mask via its data I/O port.
    unsafe fn write_interrupt_mask(&mut self, mask: u8) {
        self.data.write(mask);
    }
}

/// The two 8259 PICs, chained together.
pub struct ChainedPics {
    pics: [Pic; 2],
}

impl ChainedPics {
    /// Create an interface for the two 8259 PICs, specifying the desired interrupt offsets for both.
    ///
    /// # Interrupt Offset Note
    ///
    /// The default PIC configuration, which sends interrupt vector numbers in the range
    /// of 0 to 15, is not usable in protected mode since the numbers in that range are
    /// occupied by CPU exceptions in protected mode. If you return to real mode from
    /// protected mode, you must restore the PICs to their default configurations. You can
    /// do this using the [`ChainedPics::restore`] method.
    pub const unsafe fn new(master_offset: u8, slave_offset: u8) -> Self {
        Self {
            pics: [
                Pic::new(master_offset, MASTER_CMD, MASTER_DATA),
                Pic::new(slave_offset, SLAVE_CMD, SLAVE_DATA),
            ],
        }
    }

    /// Initialize both of the PICs.
    pub unsafe fn initialize(&mut self) {
        // We need to add a delay between writes to our PICs, especially on
        // older motherboards. But we don't necessarily have any kind of
        // timers yet, because most of them require interrupts. Various
        // older versions of Linux and other PC operating systems have
        // worked around this by writing garbage data to port 0x80, which
        // allegedly takes long enough to make everything work on most
        // hardware.
        let mut wait_port: Port<u8> = Port::new(0x80);
        let mut wait = || wait_port.write(0);

        // Send each PIC the initialization command.
        self.pics[0].command.write(PIC_INIT);
        wait();
        self.pics[1].command.write(PIC_INIT);
        wait();

        // INCOMPLETE
    }

    /// Restore the vector offsets to the defaults, which do not conflict with anything in real mode.
    pub const fn restore(&mut self) {
        todo!();
    }
}
