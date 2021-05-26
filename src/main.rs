#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(asm)]
#![feature(custom_test_frameworks)]
#![test_runner(operating_system::test_runner)]
#![reexport_test_harness_main = "test_main"]

mod std;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use x86_64::VirtAddr;

// Defines the entry point function.
//
// The function must have the signature `fn(&'static BootInfo) -> !`.
//
// This macro just creates a function named `_start`, which the linker will use as the entry
// point. The advantage of using this macro instead of providing an own `_start` function is
// that the macro ensures that the function and argument types are correct.
entry_point!(kernel_main);

/// This follows the implementation and guide of building a operating system in rust
/// by: https://os.phil-opp.com - current position: Double Faults
// noinspection RsUnresolvedReference
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Hello World{}", "!");

    operating_system::init();

    use std::memory::BootInfoFrameAllocator;

    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    // as before
    #[cfg(test)]
        test_main();

    println!("It did not crash!");
    std::interrupts::htl_loop();
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    std::interrupts::htl_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    operating_system::test_panic_handler(info)
}
