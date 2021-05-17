use core::fmt;
use spin::Mutex;
use volatile::Volatile;

/// The assigned u8 representation of the vga color assignment, this is the color that would be
/// assigned to the given text being written to the display.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    /// Returns a new ColorCode for the vga buffer, including the given background and foreground
    /// color that will be applied to the input. Using the assigned color codes. Ensuring to place
    /// the foreground color into the correct bit position.
    pub fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

/// Since the field ordering in default structs is undefined in Rust, we need the repr(C) attribute.
/// It guarantees that the struct's fields are laid out exactly like in a C struct and thus
/// guarantees the correct field ordering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenCharacter {
    ascii_character: u8,
    color_code: ColorCode,
}

/// There are modes with a character box width of 9 dots (e.g. the default 80×25 mode), however the
/// 9th column is used for spacing between characters, so the content cannot be changed. It is
/// always blank, and drawn with the current background colour
const TEXT_BUFFER_HEIGHT: usize = 25;
const TEXT_BUFFER_WIDTH: usize = 80;

/// The text buffer for the vga input, ensure to keep the same memory layout as a char array of u8
/// instead of the memory layout with the pointer information that would be set by rust.
#[repr(transparent)]
pub struct Buffer {
    chars: [[Volatile<ScreenCharacter>; TEXT_BUFFER_WIDTH]; TEXT_BUFFER_HEIGHT],
}

/// The 'static lifetime specifies that the reference is valid for the whole program run time (which
/// is true for the VGA text buffer).
pub struct Writer {
    pub column_position: usize,
    pub color_code: ColorCode,
    pub buffer: &'static mut Buffer,
}

impl Writer {
    /// Writes each byte of the input_string into the vga buffer, the input screen buffer values
    /// must be within the given of 0x20 -> 0x7e.
    ///
    /// # Arguments
    ///
    /// `input_string` The string that is going to be written to the output.
    ///
    /// # Example
    ///
    /// ```
    /// writer.write_string("Hello, World");
    /// ```
    pub fn write_string(&mut self, input_string: &str) {
        for byte in input_string.bytes() {
            match byte {
                0x20..=0x7e => self.write_byte(byte),
                b'\n' => self.new_line(),
                _ => self.write_byte(0xfe),
            }
        }
    }

    /// Writes the specified byte into the VGA buffer, if the byte is a new line then ensures to
    /// create a new line, otherwise if the buffer is going to overflow, insert a new line.
    ///
    /// # Arguments
    ///
    /// `byte` - The byte that will be written into the vga buffer.
    ///
    /// # Example
    ///
    /// ```
    /// writer::write_byte(b'\n');
    /// ```
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                // If the given column is going to overflow by meeting the max current buffer width
                // insert a new line before continuing. Otherwise continue as normal.
                if self.column_position >= TEXT_BUFFER_WIDTH {
                    self.new_line()
                }

                // TODO: Missing support for blinking?
                self.buffer.chars[TEXT_BUFFER_HEIGHT - 1][self.column_position].write(
                    ScreenCharacter {
                        ascii_character: byte,
                        color_code: self.color_code,
                    },
                );

                self.column_position += 1;
            }
        }
    }

    /// Inserts a new line at the bottom of th VGA buffer by shifting all rows up one and clearing
    /// the bottom row by inserting all spaces. Finally resetting back to the starting position.
    ///
    /// # Example
    ///
    /// ```
    /// writer.write_string("Hello, World\n");
    /// ```
    fn new_line(&mut self) {
        for row in 1..TEXT_BUFFER_HEIGHT {
            for col in 0..TEXT_BUFFER_WIDTH {
                let char = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(char);
            }
        }

        self.clear_row(TEXT_BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    /// Replaces all characters in the given row with spaces, called after a newline has been
    /// written into the buffer.
    ///
    /// # Example
    ///
    /// ```
    /// writer.write_string("Hello, World\n");
    /// ```
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenCharacter {
            ascii_character: b' ',
            color_code: self.color_code,
        };

        for col in 0..TEXT_BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

impl fmt::Write for Writer {
    /// Write a string to the VGA Buffer using the fmt::write implementation to provide formatting.
    /// This will use continue to write to vga buffer as a standard string output when formatted.
    ///
    /// # Arguments
    ///
    /// `s` The string input being written to the VGA buffer.
    ///
    /// # Example
    ///
    /// ```
    /// write!(WRITER.lock(), "some numbers: {} {}", 42, 1.23244").unwrap();
    /// ```
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::std::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    // only  lock the vga writer a time when  no interrupts
    // are occurring. This is use to stop the  chance of the
    // spin  lock causing  a dead lock between the interrupt
    // and the kernal.
    //
    // The without_interrupts function takes a closure and
    // executes it in an interrupt-free environment. We use
    // it to ensure that no interrupt can occur as long as
    // the Mutex is locked.
    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    })
}

// By introducing the writer as a global static, it begins to ensure that more problems occur. By
// having a global static means that you cannot easily have mutual exclusion. And need to
// synchronize. Mutable statics are one way but this is highly discouraged.
//
// Instead we are going to be using spin locks to provide safe interior mutability within the
// static writer.
lazy_static::lazy_static! {
     pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
   });
}

#[cfg(test)]
#[macro_use]
mod test {
    use crate::println;
    #[cfg(test)]
    use crate::std::vga_buffer::*;

    #[test_case]
    fn test_println_single_line() {
        println!("test_println_single_line output");
    }

    #[test_case]
    fn test_println_many_short() {
        for _ in 0..200 {
            println!("test_println_many_short output");
        }
    }

    #[test_case]
    fn test_println_many_long() {
        for _ in 0..200 {
            println!(
                "test_println_many_long output test_println_many_long output \
            test_println_many_long output test_println_many_long output test_println_many_long \
            output test_println_many_long output"
            );
        }
    }

    #[test_case]
    fn test_println_single_line_output() {
        use core::fmt::Write;
        use x86_64::instructions::interrupts;

        let s = "some test string that fits on a single line";
        interrupts::without_interrupts(|| {
            let mut writer = WRITER.lock();
            writeln!(writer, "\n{}", s).expect("writeln failed");
            for (i, c) in s.chars().enumerate() {
                let screen_char = writer.buffer.chars[TEXT_BUFFER_HEIGHT - 2][i].read();
                assert_eq!(char::from(screen_char.ascii_character), c);
            }
        });
    }

    #[test_case]
    fn test_println_long_should_overflow() {
        use core::fmt::Write;
        use x86_64::instructions::interrupts;

        let input =
            " Some test string that fits on a single  Some test string that fits on a single ";

        interrupts::without_interrupts(|| {
            let mut writer = WRITER.lock();

            writeln!(writer, "\n{}", input).expect("writeln failed");
            writeln!(writer, "{}", input).expect("writeln failed");
            writeln!(writer, "{}", input).expect("writeln failed");
            writeln!(writer, "").expect("writeln failed");

            for i in 1..4 {
                for (k, c) in input.chars().enumerate() {
                    // minus i for the given row position and then an additional minus 1 for the new
                    // line that would again shift all components up one row.
                    let screen_char = writer.buffer.chars[TEXT_BUFFER_HEIGHT - i - 2][k].read();
                    assert_eq!(char::from(screen_char.ascii_character), c);
                }
            }
        })
    }
}

