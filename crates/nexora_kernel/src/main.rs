#![no_std]
#![no_main]

mod boot;

use core::arch::asm;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}

#[inline(always)]
unsafe fn outb(port: u16, val: u8) {
    asm!("out dx, al", in("dx") port, in("al") val, options(nomem, nostack, preserves_flags));
}

#[inline(always)]
unsafe fn outw(port: u16, val: u16) {
    asm!("out dx, ax", in("dx") port, in("ax") val, options(nomem, nostack, preserves_flags));
}

fn serial_write_str(s: &str) {
    for byte in s.bytes() {
        unsafe {
            outb(0x3F8, byte);
        }
    }
}

#[no_mangle]
pub extern "C" fn kmain(_multiboot_info_addr: usize) -> ! {
    // Early Kernel Debug Output via Serial COM1
    serial_write_str("[NEXORAOS_BOOT_OK]\n");

    // Trigger isa-debug-exit (IO Port 0xF4) for automated CI test exit
    unsafe {
        outw(0xf4, 0x10);
    }

    // Fallback infinite halt loop
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
