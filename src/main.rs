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
        let arg = [smc::SMC_ASC_MARK_REALM, call.argument()[0], 0, 0, 0];
        let ret = smc::call(arg);
        call.reply(ret[0]);
    });

    mainloop.set_event_handler(rmi::Code::GranuleUndelegate, |call| {
        println!("RMM: requested granule undelegation");
        let arg = [smc::SMC_ASC_MARK_NONSECURE, call.argument()[0], 0, 0, 0];
        let ret = smc::call(arg);
        call.reply(ret[0]);
    });

    mainloop.set_default_handler(|call| {
        eprintln!("RMM: no proper rmi handler - code:{:?}", call.code());
    });

    mainloop.run();

    //TODO implement panic!
    eprintln!("RMM: failed to run the mainloop\n");
    panic::halt();
}
