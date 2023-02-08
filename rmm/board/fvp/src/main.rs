#![no_std]
#![no_main]
#![feature(asm_const)]
#![feature(naked_functions)]
#![warn(rust_2018_idioms)]
#![deny(warnings)]

extern crate alloc;

#[macro_use]
extern crate log;

mod driver;
mod entry;

use armv9a::allocator;
use armv9a::config;
use armv9a::cpu;
use armv9a::helper;

use monitor::listen;
use monitor::mainloop::Mainloop;
use monitor::rmi::{self, Receiver};

#[no_mangle]
pub unsafe fn main() -> ! {
    info!(
        "booted on core {:2} with EL{}!",
        cpu::get_cpu_id(),
        helper::regs::current_el()
    );

    let mut mainloop = Mainloop::new(Receiver::new());

    init_instance();
    rmi::gpt::set_event_handler(&mut mainloop);
    rmi::realm::set_event_handler(&mut mainloop);
    rmi::version::set_event_handler(&mut mainloop);

    listen!(mainloop, || {
        warn!("RMM: idle handler called.");
    });

    mainloop.run();

    panic!("failed to run the mainloop");
}

fn init_instance() {
    monitor::realm::vm::set_instance(armv9a::realm::vm::VMManager::new());
    monitor::smc::set_instance(armv9a::smc::SMC::new());
    monitor::config::set_instance(armv9a::config::RMMConfig::new());
}
