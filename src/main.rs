#![no_std]
#![no_main]
#![feature(asm)]

mod std;

static HELLO: &[u8] = b"Hello World!";

/// This follows the implementation and guide of building a operating system in rust
/// by: https://os.phil-opp.com
#[no_mangle]
pub extern "C" fn _start() -> ! {
    let vga_buffer = 0xb8000 as *mut u8;

    for (i, &byte) in HELLO.iter().enumerate() {
        unsafe {
            *vga_buffer.offset(i as isize * 2) = byte;
            *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
        }
    }

    loop {}
}
