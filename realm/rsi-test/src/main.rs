#![no_std]
#![no_main]
#![feature(asm_const)]
#![warn(rust_2018_idioms)]

mod entry;
mod mock;
mod panic;
mod stack;

#[no_mangle]
pub unsafe fn main() -> ! {
    mock::get_ns_buffer();
    mock::exit_to_host();

    loop {}
}
