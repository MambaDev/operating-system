use core::panic::PanicInfo;
use crate::println;

#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
