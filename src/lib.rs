#![no_std]
#![feature(abi_x86_interrupt)]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(const_fn_trait_bound)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

#[cfg(test)]
use bootloader::{entry_point, BootInfo};

pub mod std;

#[cfg(test)]
entry_point!(test_kernel_main);

// Entry point for `cargo xtest`
// noinspection RsUnresolvedReference
#[cfg(test)]
fn test_kernel_main(_boot_info: &'static BootInfo) -> ! {
    init();
    test_main();
    std::interrupts::htl_loop();
}

pub fn init() {
    std::gdt::init();
    std::interrupts::init_idt();

    // init the PIC controllers. These are unsafe since it could
    // cause unexpected output if the given PIC controllers are
    // misconfigured.
    unsafe { std::interrupts::PICS.lock().initialize() };

    // Enable interrupts to be processed by the CPU. Meaning that
    // now the CPU does listen ot the interrupt controller. Executing
    // a special "sti" instruction "set interrupt" to enable external
    // interrupts.
    //
    // At this point we must configure the basic hardware timer
    // (intel 8253) since its enabled by default otherwise we will
    // start getting double faults.
    x86_64::instructions::interrupts::enable()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
#[allow(dead_code)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

#[allow(dead_code)]
pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    std::interrupts::htl_loop();
}

// During the testing, we will be exporting all our testing output to the serial port for the
// virtual machine, using this output to read the results of the tests in the console. And thus
// if a test panics, we will need the output, this writes to the serial port, not the screen.
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}
