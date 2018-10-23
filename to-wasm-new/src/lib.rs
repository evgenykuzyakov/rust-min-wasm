// We aren't using the standard library.
#![no_std]

use core::panic::PanicInfo;

#[no_mangle]
pub fn add_one(x: i32) -> i32 {
    x + 1
}

#[panic_handler]
#[no_mangle]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}