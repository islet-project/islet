#![no_std]
#![no_main]
#![feature(const_fn)]
#![feature(const_fn_fn_ptr_basics)]
#![feature(const_mut_refs)]
#![feature(llvm_asm)]
#![feature(alloc_error_handler)]
#![warn(rust_2018_idioms)]

pub mod alloc;
pub mod config;
pub mod driver;
pub mod entry;
pub mod panic;
pub mod rmi;

use realm_management_monitor::io::{stdout, Write};

#[no_mangle]
#[allow(unused)]
pub unsafe fn main() -> ! {
    let _ = stdout().write_all("RMM: booted on core!\n".as_bytes());

    loop {
        rmi::rmm_exit();
        let _ = stdout().write_all("RMM: invoked!\n".as_bytes());
    }
}
