use core::fmt::Arguments;
use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::SerialPort;

// Globally accessible implementation of the first serial port of the virtual machine. This will
// be used during testing to ensure that the tests can run headless and process the output to the
// virtual machines terminal output. Using a spin lock to ensure mutual exclusion.
lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

/// Prints to the host through the serial interface.
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::std::serial::_print(format_args!($($arg)*));
    };
}

/// Prints to the host through the serial interface, appending a newline.
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}

#[doc(hidden)]
pub fn _print(args: Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    // The without_interrupts function takes a closure and
    // executes it in an interrupt-free environment. We use
    // it to ensure that no interrupt can occur as long as
    // the Mutex is locked.
    interrupts::without_interrupts(|| {
        SERIAL1
            .lock()
            .write_fmt(args)
            .expect("Printing to serial failed");
    })
}
