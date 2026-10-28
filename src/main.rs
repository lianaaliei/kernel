#![no_std]
#![no_main]

mod vga;

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    vga::clear();
    println!("Hello from my own kernel{}", "!");
    println!("Stage 1: VGA text output works.");

    loop {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("\nKERNEL PANIC: {}", info);
    loop {
        core::hint::spin_loop();
    }
}
