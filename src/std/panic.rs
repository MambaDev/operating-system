use core::panic::PanicInfo;

#[cfg(not(test))]
use crate::{println};

#[cfg(test)]
use crate::{serial_println};

#[cfg(test)]
use crate::test::{QemuExitCode, exit_qemu};

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

// During the testing, we will be exporting all our testing output to the serial port for the
// virtual machine, using this output to read the results of the tests in the console. And thus
// if a test panics, we will need the output, this writes to the serial port, not the screen.
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}