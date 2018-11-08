// We aren't using the standard library.
#![no_std]
#![feature(alloc_error_handler)]
#![feature(alloc)]
#![feature(allocator_api)]

use core::panic::PanicInfo;

#[allow(unused)]
#[macro_use]
extern crate alloc;

extern crate wee_alloc;

extern crate byteorder;

use byteorder::{LittleEndian, ByteOrder};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

extern "C" {
    fn db_put(key: *const u8, value: *const u8, len: u32); 
    fn db_get(key: *const u8, value: *mut u8, max_len: u32) -> u32; 
}

#[no_mangle]
fn key_to_str(key: u32) -> [u8; 16] {
    let mut str_key = [0u8; 16];
    str_key[..15].clone_from_slice(&b"key: 0000000000"[..]);
    let mut pos = str_key.len() - 1;
    str_key[pos] = 0u8;
    pos -= 1;
    let mut mkey = key; 
    while mkey > 0 { 
        str_key[pos] = b'0' as u8 + (mkey % 10) as u8;
        pos -= 1;
        mkey /= 10;
    }
    str_key
}

#[no_mangle]
pub fn put_int(key: u32, value: i32) {
    let mut val_bytes = [0u8; 4];
    LittleEndian::write_i32(&mut val_bytes, value);
    unsafe {
        db_put(key_to_str(key).as_ptr(), val_bytes.as_ptr(), 4);
    }
}

#[no_mangle]
pub fn get_int(key: u32) -> i32 {
    let mut dst = [0u8; 4];
    unsafe {
        let read_size = db_get(key_to_str(key).as_ptr(), dst.as_mut_ptr(), 4);
        assert!(read_size == 4);
        LittleEndian::read_i32(&dst)
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[alloc_error_handler]
fn foo(_: core::alloc::Layout) -> ! {
  loop {}
}