#![no_std]
#![no_main]
#![feature(asm_const)]
#![warn(rust_2018_idioms)]

mod entry;
mod panic;

#[no_mangle]
pub unsafe fn main() -> ! {
    let mut i: usize = 0;
    loop {
        core::arch::asm! {
            "mov x0, {}",
            "hvc #0",
            in(reg) i,
        };
        i += 1;
    }
}
