#![no_std]
#![no_main]
#![feature(llvm_asm)]
#![feature(naked_functions)]

extern crate alloc;

#[macro_use]
extern crate log;

mod driver;
mod entry;

use armv9a::allocator;
use armv9a::config;
use armv9a::cpu;
use armv9a::helper;
use armv9a::rmi;

use monitor::listen;
use monitor::mainloop::Mainloop;

#[no_mangle]
pub unsafe fn main() -> ! {
    info!(
        "booted on core {:2} with EL{}!",
        cpu::get_cpu_id(),
        helper::regs::current_el()
    );

    let mut mainloop = Mainloop::new(rmi::Receiver::new());

    rmi::gpt::set_event_handler(&mut mainloop);
    rmi::realm::set_event_handler(&mut mainloop);
    rmi::version::set_event_handler(&mut mainloop);

    listen!(mainloop, || {
        warn!("RMM: idle handler called.");
    });

    mainloop.run();

    panic!("failed to run the mainloop");
}
