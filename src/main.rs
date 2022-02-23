#![no_std]
#![no_main]
#![feature(const_fn)]
#![feature(const_fn_fn_ptr_basics)]
#![feature(const_mut_refs)]
#![feature(llvm_asm)]
#![feature(alloc_error_handler)]
#![feature(naked_functions)]
#![feature(global_asm)]
#![warn(rust_2018_idioms)]

pub mod aarch64;
pub mod allocator;
pub mod config;
pub mod cpu;
pub mod driver;
pub mod entry;
pub mod panic;
pub mod rmi;
pub mod smc;
pub mod traps;

extern crate alloc;

use realm_management_monitor::communication::Event;
use realm_management_monitor::io::Write as IoWrite;
use realm_management_monitor::mainloop::Mainloop;
use realm_management_monitor::{eprintln, println};

#[no_mangle]
#[allow(unused)]
pub unsafe fn main() -> ! {
    println!("RMM: booted on core {:?}!", cpu::id());

    let mut mainloop = Mainloop::new(rmi::Receiver::new());

    mainloop.set_event_handler(rmi::Code::Version, |call| {
        println!("RMM: requested version information");
        call.reply(config::ABI_VERSION);
    });

    mainloop.set_event_handler(rmi::Code::GranuleDelegate, |call| {
        println!("RMM: requested granule delegation");
        let cmd = usize::from(smc::Code::MarkRealm);
        let arg = [call.argument()[0], 0, 0, 0];
        let ret = smc::call(cmd, arg);
        call.reply(ret[0]);
    });

    mainloop.set_event_handler(rmi::Code::GranuleUndelegate, |call| {
        println!("RMM: requested granule undelegation");
        let cmd = usize::from(smc::Code::MarkNonSecure);
        let arg = [call.argument()[0], 0, 0, 0];
        let ret = smc::call(cmd, arg);
        call.reply(ret[0]);
    });

    mainloop.set_default_handler(|call| {
        eprintln!("RMM: no proper rmi handler - code:{:?}", call.code());
    });

    println!("CurrentEL is {}", crate::aarch64::regs::current_el());
    // crate::aarch64::asm::brk(10);
    mainloop.run();

    panic!("failed to run the mainloop");
}
