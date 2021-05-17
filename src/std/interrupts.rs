use crate::std::gdt;
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use crate::print;
use crate::println;

// The index values in which will be used in the interrupt
// descriptor table to allow the CPU to know which handler
// to be called for external interrupts.
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    // The offset in which the timer interrupt is triggered.
    Timer = PIC_1_OFFSET,
    // The offset in which the keyboard interrupt is triggered.
    Keyboard = PIC_1_OFFSET + 1,
    SerialPortOne = PIC_1_OFFSET + 3,
    SerialPortTwo = PIC_1_OFFSET + 4,
    ParallelPortTwoAndThree = PIC_1_OFFSET + 5,
    FloppyDisk = PIC_1_OFFSET + 6,
    ParallelPortOne = PIC_1_OFFSET + 7,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        self as usize
    }
}

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

// We're setting the offsets for the pics to the range 32–47 as we noted above.
// By wrapping the ChainedPics struct in a Mutex we are able to get safe mutable
// access (through the lock method),
//
// allowing us to communicate with the PICS, setup which interrupt
// handlers are going to be executed when the CPU triggers them and
// then finally telling the CPU that the interrupt has been handled.
pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);

        // configure the double fault handler with the
        // alternative stack to ensure double faults
        // don't cause triple faults via stack overflows.
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }

        // configure the timer interrupt.
        idt[InterruptIndex::Timer.as_usize()]
            .set_handler_fn(timer_interrupt_handler);

        idt
    };
}

#[allow(dead_code)]
pub fn init_idt() {
    IDT.load();
}

/// Exception Type
///
/// Faults: These can be corrected and the program may continue as if nothing happened.
/// Traps: Traps are reported immediately after hte execution of the trapping instruction.
/// Aborts: Some severe unrecoverable errors.
///
/// reference: https://wiki.osdev.org/Exceptions

/// Handler for processing a break point exception triggered by the x86 CPU.
/// The vector number is 0x3 which is of type TRAP.
///
/// # Arguments
///
/// `stack_frame` The stack frame at the point of which the breakpoint was hit.
///
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame)
}

/// Handler for processing double fault exceptions.
///
/// Double fault exceptions can occur when a second exception occurs during the handling of
/// a prior (first) exception handler. The can is important: Only a very specific combination
/// of exceptions lead to a double fault. These combinations are:
///
/// Fist Exception:
///
/// Divide-by-zero,
/// Invalid TSS,
/// Segment Not Present,
/// Stack-Segment Fault,
/// General Protection Fault
///
/// Second Exception:
///
/// Invalid TSS,
/// Segment Not Present,
/// Stack-Segment Fault,
/// General Protection Fault
///
/// First Exception:
///
/// Page Fault
///
/// Second Exception:
///
/// Page Fault,
/// Invalid TSS,
/// Segment Not Present,
/// Stack-Segment Fault,
/// General Protection Fault
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame)
}

#[test_case]
fn test_breakpoint_exception() {
    // invoke th break point execution, if it executes and does not fail
    // then we have passed since it should not fault.
    x86_64::instructions::interrupts::int3();
}
/// Handler for processing timer interrupts.
///
extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    print!(".");

    // Let the PICS know that the interrupt has been handled via
    // EOI (end of interrupt). If not done, the PIC will assume
    // we are still busy and wait before sending the next one.
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8())
    }
}
