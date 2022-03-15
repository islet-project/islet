#![no_std]
#![no_main]
#![feature(const_fn)]
#![feature(const_fn_fn_ptr_basics)]
#![feature(const_mut_refs)]
#![feature(llvm_asm)]
#![feature(alloc_error_handler)]
#![feature(naked_functions)]
#![feature(global_asm)]
#![feature(specialization)]
#![warn(rust_2018_idioms)]

pub mod aarch64;
pub mod allocator;
pub mod config;
pub mod driver;
pub mod entry;
pub mod panic;
pub mod realm;
pub mod rmi;
pub mod smc;

extern crate alloc;

#[macro_use(bitflags)]
extern crate bitflags;

use rmm_core::communication::Event;
use rmm_core::io::Write as IoWrite;
use rmm_core::mainloop::Mainloop;
use rmm_core::{eprintln, println};

#[no_mangle]
#[allow(unused)]
pub unsafe fn main() -> ! {
    println!("RMM: booted on core {:?}!", aarch64::cpu::id());

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

    mainloop.run();

    panic!("failed to run the mainloop");
}
