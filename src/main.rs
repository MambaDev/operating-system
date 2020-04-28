#![no_std]
#![no_main]
#![feature(asm)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test::test_runner)]
#![reexport_test_harness_main = "test_main"]

mod std;
mod test;

/// This follows the implementation and guide of building a operating system in rust
/// by: https://os.phil-opp.com
// noinspection RsUnresolvedReference
#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");

    #[cfg(test)]
        test_main();

    loop {}
}
