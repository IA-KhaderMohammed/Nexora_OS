//! Early Kernel Serial Debug Console
//!
//! Architecture Note: This is an early debug logging facility, NOT a full user-space driver.
//! In NexoraOS Architecture v1.2.4, full device drivers reside strictly in User Space.
//!
//! Technical Specification:
//! - Device: 16550 UART (COM1 at base I/O port 0x3F8)
//! - Baud Rate: 38400 baud (Baud Divisor = 3 with standard 1.8432 MHz clock: 115200 / 3)
//! - Operating Mode: Polling-based, Output-only
//! - Concurrency: Not safe for concurrent multi-CPU or multi-threading (no locks/Mutex)

use core::arch::asm;
use core::fmt;

const COM1_PORT: u16 = 0x3F8;

pub struct SerialPort {
    port: u16,
}

impl SerialPort {
    pub const fn new(port: u16) -> Self {
        Self { port }
    }

    /// Safe initialisation interface wrapping internal hardware port configuration.
    pub fn init(&mut self) {
        unsafe {
            outb(self.port + 1, 0x00); // Disable all interrupts
            outb(self.port + 3, 0x80); // Enable DLAB (Set Baud Rate Divisor)
            outb(self.port + 0, 0x03); // Divisor LSB = 3 (38400 baud)
            outb(self.port + 1, 0x00); // Divisor MSB = 0
            outb(self.port + 3, 0x03); // Disable DLAB; Protocol: 8 bits, no parity, 1 stop bit
            outb(self.port + 2, 0xC7); // Enable FIFO, clear queues, 14-byte threshold
            outb(self.port + 4, 0x0B); // Set IRQs enabled, RTS/DSR set
        }
    }

    /// Sends a single byte using polling on LSR bit 5.
    pub fn send_byte(&mut self, byte: u8) {
        unsafe {
            while (inb(self.port + 5) & 0x20) == 0 {}
            outb(self.port, byte);
        }
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            if byte == b'\n' {
                self.send_byte(b'\r');
            }
            self.send_byte(byte);
        }
        Ok(())
    }
}

// Encapsulated assembly wrappers for low-level I/O operations
#[inline(always)]
unsafe fn outb(port: u16, val: u8) {
    asm!(
        "out dx, al",
        in("dx") port,
        in("al") val,
        options(nomem, nostack, preserves_flags)
    );
}

#[inline(always)]
unsafe fn inb(port: u16) -> u8 {
    let ret: u8;
    asm!(
        "in al, dx",
        out("al") ret,
        in("dx") port,
        options(nomem, nostack, preserves_flags)
    );
    ret
}

pub static mut SERIAL1: SerialPort = SerialPort::new(COM1_PORT);

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    unsafe {
        SERIAL1.write_fmt(args).unwrap();
    }
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($($arg:tt)*) => ($crate::serial_print!("{}\n", format_args!($($arg)*)));
}
