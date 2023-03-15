#![no_std]
#![no_main]
#![feature(asm_const)]
#![feature(naked_functions)]
#![warn(rust_2018_idioms)]

#[macro_use]
extern crate log;

mod entry;

#[no_mangle]
pub unsafe fn main() -> ! {
    loop {
        // core::arch::asm!("hvc #0x0");
        info!("Launched on normal world.");
    }
}
