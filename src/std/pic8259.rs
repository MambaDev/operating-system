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