#![no_std]
#![no_main]
#![feature(asm)]


use crate::std::vga_buffer::*;

mod std;

/// This follows the implementation and guide of building a operating system in rust
/// by: https://os.phil-opp.com
#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello, World");
    println!("I'm writing to the VGA buffer");
    panic!("I'm panicing!!");

    loop {}
}
