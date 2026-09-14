#![no_std]
#![no_main]

mod serial;

use core::arch::asm;
use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn kmain(multiboot_info_addr: usize) -> ! {
    // Initialize Early Kernel Serial Debug Console (COM1)
    unsafe {
        serial::SERIAL1.init();
    }

    // Verify formatted serial output with explicit expected tokens
    serial_println!("[NEXORAOS_BOOT_OK]");
    serial_println!("Multiboot2 Info Addr: 0x{:x}", multiboot_info_addr);
    serial_println!("Console Test: status={}", 200);

    // QEMU isa-debug-exit request
    unsafe {
        asm!(
            "out dx, ax",
            in("dx") 0xf4u16,
            in("ax") 0x10u16,
            options(nomem, nostack, preserves_flags)
        );
    }

    loop {
        unsafe {
            asm!("hlt");
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("KERNEL PANIC: {}", info);
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
