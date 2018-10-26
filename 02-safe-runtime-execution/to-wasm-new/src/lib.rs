// We aren't using the standard library.
#![no_std]

use core::panic::PanicInfo;

// Recursion that might cause stack overflow
#[no_mangle]
pub fn rec(x: i32, n: i32) -> i32 {
    match x {
        _ if x < n => x + rec(x + 1, n),
        _ => x,
    }
}

#[panic_handler]
#[no_mangle]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}