#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(asm)]
#![feature(custom_test_frameworks)]
#![test_runner(operating_system::test_runner)]
#![reexport_test_harness_main = "test_main"]

mod std;

use core::panic::PanicInfo;

// WILO: Creating a TSS
// https://os.phil-opp.com/double-fault-exceptions/#creating-a-tss

/// This follows the implementation and guide of building a operating system in rust
/// by: https://os.phil-opp.com - current position: Double Faults
// noinspection RsUnresolvedReference
#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");

    // as before
    #[cfg(test)]
        test_main();

    println!("It did not crash!");
    loop {}
}


/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    operating_system::test_panic_handler(info)
}