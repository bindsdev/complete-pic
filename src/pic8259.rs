//! 8259 PIC interface
//!
//! # What is the 8259 PIC?
//! The 8259 PIC (or Programmable Interrupt Controller) is a crucial device in the x86 architecture.
//! Its purpose is to manage hardware interrupts by sending them to the appropriate system interrupt, allowing the system to respond to hardware devices
//! without losing time. The 8259 PIC makes handling hardware interrupts much easier. Without it, the hardware devices would have to be manually polled
//! to check if they have anything they want to do. Time would be wasted trying to go to the hardware device and figure out what it wants to do.
//!
//! ## How does the 8259 PIC work?
//! In a modern system, there are 2 8259 PICs, each with 8 inputs. The first is called the "master" while the other is called the "slave". If any input on the PIC is raised,
//! it will signal that the input needs servicing by setting an internal bit. The PIC then does two checks. It checks whether that input is masked or not, and whether another interrupt
//! is already pending. If the input is unmasked and no interrupt is pending, the PIC will raise the interrupt line. The slave PIC will send the IRQ number to the master so it can connect to
//! the interrupt line. The master PIC will then check which PIC is responsible for answering the interrupts. If the master PIC is responsible, it will send the interrupt number to the processor.
//! If the slave PIC is responsible, it will ask the slave to send the interrupt number to the processor. The PIC that answers then looks for the "vector offset" and adds it to the input line to
//! compute the interrupt number. The processor then acts on that interrupt number.
//!
//! ## The 8259 PIC today
//! The 8259 PIC is now a legacy device that has been replaced by the APIC (Advanced Programmable Interrupt Controller). The APIC is usable within multiprocessor systems.
//! The APIC can do more sophisticated and complex things than the 8259 PIC.
//!
//! ## Where can I read more?
//! The following links are useful to learning more about the 8259 PIC and interrupts:
//! - [8259 PIC on OSDev Wiki](https://wiki.osdev.org/PIC)
//! - [8259 PIC on Wikipedia](https://en.wikipedia.org/wiki/Intel_8259)
//! - [Interrupts](https://wiki.osdev.org/IRQ)
//!
//! # Public API
//!
//! This module is based off of the already existing [pic8259](https://github.com/rust-osdev/pic8259) crate. Many of the public functions are marked as `unsafe` because it is
//! very easy to cause undefined behavior by passing incorrect values that misconfigure the 8259 PIC or using the 8259 PIC incorrectly.
//!
//! # Usage
//!
//! Before utilizing this module, it is recommended that you wrap the `ChainedPics` struct in a `Mutex` to get safe mutable access to it. This can be done by using the `spin` crate.
//! Make sure to add the `spin` crate to your `Cargo.toml` under `[dependencies]`. It should look like this:
//!
//! ```toml
//! [dependencies]
//! complete_pic = { version = "0.1.0", default-features = false, features = ["8259pic"] }
//! spin = "0.9.6"
//! ```
//!
//! Next, declare a `spin::Mutex<ChainedPics>` in a `static` variable:
//!
//! ```rust
//! use complete_pic::pic8269::ChainedPics;
//! use spin::Mutex;
//!
//! const PIC1_OFFSET: u8 = 32;
//! const PIC2_OFFSET: u8 = PIC1_OFFSET + 8;
//!
//! // Map PIC interrupts to 0x20 through 0x2f.
//! static PICS: Mutex<ChainedPics> = Mutex::new(unsafe { ChainedPics::new(PIC1_OFFSET, PIC2_OFFSET) });
//! ```
//!
//! Next, initialize the PICs (make sure interrupts are disabled):
//!
//! ```rust
//! unsafe { PICS.lock().initialize(); }
//! # enable interrupts after initializing the PIC
//! ```
//!
//! When you have finished handling an interrupt, call [`ChainedPics::notify_end_of_interrupt`]. Here is an example:
//!
//! ```rust
//! #![feature(abi_x86_interrupt)]
//!
//! extern "x86-interrupt" fn timer_interrupt_handler(...) {
//!    # code
//!
//!    unsafe {
//!        # The Intel Programmable Interval Time (PIT) uses the first IRQ index.
//!        PICS.lock().notify_end_of_interrupt(PIC1_OFFSET);
//!    }  
//! }
//! ```
//!
//! # Note
//!
//! Some bootloaders might mask all the IRQs from the 8259 (legacy) PIC, like Limine. Make sure you check the bootloader's documentation before
//! you become confused due to this module not functioning as expected.

use x86_64::instructions::port::Port;

/// The command I/O port of the master PIC.
const MASTER_CMD: u16 = 0x20;

/// The data I/O port of the master PIC.
const MASTER_DATA: u16 = 0x21;

/// The command I/O port of the slave PIC.
const SLAVE_CMD: u16 = 0xA0;

/// The data I/O port of the slave PIC.
const SLAVE_DATA: u16 = 0xA1;

/// PIC initialization command.
const PIC_INIT: u8 = 0x11;

/// PIC End of Interrupt command.
const PIC_EIO: u8 = 0x20;

/// The PIC 8086 mode.
const PIC_MODE_8086: u8 = 0x01;

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
    const fn new(offset: u8, command: u16, data: u16) -> Self {
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
    /// # Safety
    ///
    /// It is important to pass the correct offsets. The default PIC configuration, which
    /// sends interrupt vector numbers in the range of 0 to 15, is not usable in
    /// protected mode since the numbers in that range are occupied by CPU exceptions in protected mode.
    /// If you return to real mode from protected mode (for whatever reason), you must restore the PICs to their
    /// default configurations.
    pub const unsafe fn new(master_offset: u8, slave_offset: u8) -> Self {
        Self {
            pics: [
                Pic::new(master_offset, MASTER_CMD, MASTER_DATA),
                Pic::new(slave_offset, SLAVE_CMD, SLAVE_DATA),
            ],
        }
    }

    /// Initialize both of the PICs. You can read more about the initialization process by checking out
    /// the following links:
    ///
    /// - <https://k.lse.epita.fr/internals/8259a_controller.html>
    /// - <https://www.eeeguide.com/8259-programmable-interrupt-controller>
    /// - <https://www.thesatya.com/8259.html>
    ///
    /// # Safety
    ///
    /// Please read the Safety section of [`ChainedPics::new`].
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

        // Save the original interrupts masks.
        let saved_masks = self.read_interrupt_masks();

        // Send each PIC the initialization command. This tells the PICs that
        // a 3-byte initialization sequence will be sent to its data port.
        self.pics[0].command.write(PIC_INIT);
        wait();
        self.pics[1].command.write(PIC_INIT);
        wait();

        // Byte 1: Setup the base offsets.
        self.pics[0].data.write(self.pics[0].offset);
        wait();
        self.pics[1].data.write(self.pics[1].offset);
        wait();

        // Byte 2: Configure chaining between the two PIC chips.
        self.pics[0].data.write(4);
        wait();
        self.pics[1].data.write(2);
        wait();

        // Byte 3: Set the PIC mode.
        self.pics[0].data.write(PIC_MODE_8086);
        wait();
        self.pics[1].data.write(PIC_MODE_8086);
        wait();

        // Restore the saved masks.
        self.write_interrupt_masks(saved_masks[0], saved_masks[1]);
    }

    /// Read the interrupt masks of both PICs.
    pub unsafe fn read_interrupt_masks(&mut self) -> [u8; 2] {
        [
            self.pics[0].read_interrupt_mask(),
            self.pics[1].read_interrupt_mask(),
        ]
    }

    /// Write to the interrupt masks of both PICs.
    pub unsafe fn write_interrupt_masks(&mut self, master_mask: u8, slave_mask: u8) {
        self.pics[0].write_interrupt_mask(master_mask);
        self.pics[1].write_interrupt_mask(slave_mask);
    }

    /// Disable both PICs by masking all interrupts.
    pub unsafe fn disable(&mut self) {
        self.write_interrupt_masks(u8::MAX, u8::MAX);
    }

    /// Check if the master or slave PIC handles the IRQ specified by the given ID.
    pub fn handles_interrupt(&self, irq_id: u8) -> bool {
        self.pics.iter().any(|p| p.handles_interrupt(irq_id))
    }

    /// Figure out which, if any, PIC in the chain needs to know about this interrupt. If the IRQ originated
    /// from the master PIC, we only need to send the EOI command to the master PIC. Otherwise, the EOI
    /// command needs to be sent to both PICs in the chain.
    ///
    /// # Safety
    ///
    /// It is important to pass the correct interrupt vector number. If the incorrect interrupt vector number
    /// is passed, it can lead to deleting an unsent interrupt or a hanging system.    
    pub unsafe fn notify_end_of_interrupt(&mut self, irq_id: u8) {
        if self.handles_interrupt(irq_id) {
            if self.pics[1].handles_interrupt(irq_id) {
                self.pics[1].end_of_interrupt();
            }

            self.pics[0].end_of_interrupt();
        }
    }

    /// Restore the vector offsets to the defaults, which do not conflict with anything in real mode.
    #[doc(hidden)]
    pub fn restore(&mut self) {
        self.pics[0].offset = 0x00;
        self.pics[1].offset = 0x08;
    }
}
