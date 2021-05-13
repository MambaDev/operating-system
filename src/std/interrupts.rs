use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use lazy_static::lazy_static;

use crate::println;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.double_fault.set_handler_fn(double_fault_handler);
        idt
    };
}

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
extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, error_code: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame)
}


#[test_case]
fn test_breakpoint_exception() {
    // invoke th break point execution, if it executes and does not fail
    // then we have passed since it should not fault.
    x86_64::instructions::interrupts::int3();
}

