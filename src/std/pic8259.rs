// Support for the 8259 Programmable Interrupt Controller, which handles
// basic I/O interrupts.  In multicore mode, we would apparently need to
// replace this with an APIC interface.
//
// The basic idea here is that we have two PIC chips, PIC1 and PIC2, and
// that PIC2 is slaved to interrupt 2 on PIC 1.  You can find the whole
// story at http://wiki.osdev.org/PIC (as usual).  Basically, our
// immensely sophisticated modern chipset is engaging in early-80s
// cosplay, and our goal is to do the bare minimum required to get
// reasonable interrupts.
//
// The most important thing we need to do here is set the base "offset"
// for each of our two PICs, because by default, PIC1 has an offset of
// 0x8, which means that the I/O interrupts from PIC1 will overlap
// processor interrupts for things like "General Protection Fault".  Since
// interrupts 0x00 through 0x1F are reserved by the processor, we move the
// PIC1 interrupts to 0x20-0x27 and the PIC2 interrupts to 0x28-0x2F.  If
// we wanted to write a DOS emulator, we'd presumably need to choose
// different base interrupts, because DOS used interrupt 0x21 for system
// calls.

#![feature(const_fn)]
#![no_std]

extern crate cpuio;

// Command sent ot begin PIC initialization.
const CMD_INIT: u8 = 0x11;

// Command sent to acknowledge and interrupt.
const CMD_END_OF_INTERRUPT: u8 = 0x20;

// The mode in which we want to run our PICs.
const MODE_8086: u8 = 0x01;

/// An individual PIC chip. This is not exported, because we always access
/// it through `Pics` below.
struct Pic {
    /// the base offset to which our interrupts are mapped.
    offset: u8,

    /// The preprocessor I/O  port on which we send commands.
    command: cpuio::UnsafePort<u8>,

    /// The preprocessor I/O port on which we send and receive data.
    data: cpuio::UnsafePort<u8>,
}

impl Pic {
    /// Are we in change of handling the specific interrupt?
    /// (Each PIC handles 8 interrupts.)
    fn handles_interrupt(&self, interrupt_id: u8) -> bool {
        self.offset <= interrupt_id && interrupt_id < self.offset + 8
    }

    /// Notify us that an interrupt has been handled and that we're ready
    /// for more.
    unsafe fn end_of_interrupt(&mut self) {
        self.command.write(CMD_END_OF_INTERRUPT);
    }
}

/// A pair of chained PIC controllers. This is the standard setup on x86.
pub struct ChainedPics {
    pics: [Pic; 2],
}

impl ChainedPics {
    /// Create a new interface for the standard PIC1 and PIC2 controllers,
    /// specifying he desired interrupt offset.
    pub const unsafe fn new(offset_one: u8, offset_two: u8) -> ChainedPics {
        ChainedPics {
            pics: [
                Pic {
                    offset: offset_one,
                    command: cpuio::UnsafePort::new(0x20),
                    data: cpuio::UnsafePort::new(0x21),
                },
                Pic {
                    offset: offset_one,
                    command: cpuio::UnsafePort::new(0x20),
                    data: cpuio::UnsafePort::new(0x21),
                },
            ],
        }
    }

    /// Initialize both our PICs. We initialize them together, at the same time
    /// because it's traditional to do so, and because I/O operations might not
    /// be instantaneous on older processors.
    pub unsafe fn initialize(&mut self) {
        // We need to add a delay between writes to our PICs, especially on
        // older motherboards. But we don't necessarily have any kind of timers yet,
        // because most of them require interrupts. Various older versions of linux
        // and other PC operating systems have worked around this by writing garbage
        // data to port 0x80. Which allegedly takes long enough to make everything
        // work on most hardware. Here, `wait` is a closure.
        let mut wait_point: cpuio::Port<u8> = cpuio::Port::new(0x80);
        let mut wait = || wait_point.write(0);

        // Save our original interrupt masks, because I'm too lazy to
        // figure out reasonable values. We'll restore these when we're
        // done.
        let saved_mask_one = self.pics[0].data.read();
        let saved_mask_two = self.pics[1].data.read();

        // Simple action to write bytes to the pics in order with a
        // execution wait in between each action.
        let write_pics = |first: u8, second: u8| {
            self.pics[0].command.write(first);
            wait();
            self.pics[1].command.write(second);
            wait();
        };

        // Tell each PIC that we're going to send it a three-byte
        // initialization sequence on its data port.
        write_pics(CMD_INIT, CMD_INIT);

        // Byte 1: Set up our base offsets.
        write_pics(self.pics[0].offset, self.pics[1].offset);

        // Byte 2: Configure chaining between PIC1 and PIC2.
        write_pics(4, 2);

        // Byte 3: Set our mode
        write_pics(MODE_8086, MODE_8086);

        // Restore our saved masks.
        self.pics[0].command.write(saved_mask_one);
        self.pics[1].command.write(saved_mask_two);
    }

    // Do we need to handle this kind of interrupt for our pics?
    pub fn handles_interrupt(&self, interrupt: u8) -> bool {
        self.pics.iter().any(|p| p.handles_interrupt(interrupt_id))
    }

    /// Figure out which (if any) PICs in our chain need to know about this
    /// interrupt.  This is tricky, because all interrupts from `pics[1]`
    /// get chained through `pics[0]`.
    pub unsafe fn notify_end_of_interrupt(&mut self, interrupt_id: u8) {
        if self.handles_interrupt(interrupt_id) {
            if self.pics[1].handles_interrupt(interrupt_id) {
                self.pics[1].end_of_interrupt();
            }
            self.pics[0].end_of_interrupt();
        }
    }
}
