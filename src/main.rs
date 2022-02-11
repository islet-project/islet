#![no_std]
#![no_main]
#![feature(const_fn)]
#![feature(const_fn_fn_ptr_basics)]
#![feature(const_mut_refs)]
#![feature(llvm_asm)]
#![feature(alloc_error_handler)]
#![feature(naked_functions)]
#![warn(rust_2018_idioms)]

pub mod allocator;
pub mod config;
pub mod cpu;
pub mod driver;
pub mod entry;
pub mod panic;
pub mod rmi;
pub mod smc;

extern crate alloc;

use realm_management_monitor::communication::{Event, Receiver};
use realm_management_monitor::io::Write as IoWrite;
use realm_management_monitor::{eprintln, println};

#[no_mangle]
#[allow(unused)]
pub unsafe fn main() -> ! {
    println!("RMM: booted on core {:?}!", cpu::id());

    let receiver = rmi::Receiver::new();
    for call in receiver.iter() {
        eprintln!("RMM: no proper rmi handler - code:{:?}", call.code());
    }

    //TODO implement panic!
    eprintln!("RMM: failed to receive RMI\n");
    panic::halt();
}
