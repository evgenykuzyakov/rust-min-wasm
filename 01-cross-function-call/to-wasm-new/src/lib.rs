// We aren't using the standard library.
#![no_std]

use core::panic::PanicInfo;

// Define external functions and types here
extern {
    fn unused_fn(x: f32) -> f32;
    fn mult_two(x: i32) -> i32;
}

// This function would call extern function
#[no_mangle]
pub fn mult_four(x: i32) -> i32 {
    // Has to be unsafe since rustc can't verify the external function
    unsafe {
        mult_two(mult_two(x))
    }
}

// We are not going to call it
#[no_mangle]
pub fn wasm_unused_fn(x: f32) -> f32 {
    unsafe {
        unused_fn(x)
    }
}

#[panic_handler]
#[no_mangle]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}