
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

// Implemented custom wrapper for executing tests within rust. Since its currently a custom os
// and the standard lib does contain the testing framework, this framework is not included. And
// thus requires us to implement our own testing framework.
#[cfg(test)]
#[allow(dead_code)]
pub fn test_runner(tests: &[&dyn Fn()]) {
    use crate::serial_println;
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }

    exit_qemu(QemuExitCode::Success)
}

